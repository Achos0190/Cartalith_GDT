//! Address classification: `classify_host`, a pure function of a host string.
//!
//! WHY: Ruling DF (`LARGE_ITEM_RULINGS.md`) and `LLM_ASSISTANT_SCOPE.md` s.5a
//! hang two decisions on *where the language-model server is*: which of the
//! "Local providers" / "Remote providers" switches a base URL needs, and
//! whether Cartalith scales its compute back to the lowest-power GPU so the
//! dedicated card is left to a model running on the same machine. Both read
//! the base-URL host **only**. This module is that reading, kept pure so the
//! whole table can be tested without a network or an operating system.
//!
//! NO DNS, NO SOCKETS. A hostname is classified by its text alone and is never
//! resolved (Ruling DF rule 4): `host.docker.internal` and `myhost.local`
//! come back [`HostClass::Unknown`], never guessed. `core::net` is used for
//! its address *parsers* only; the socket half of the standard library lives in a
//! different module, which `tests/no_network.rs` forbids this crate to mention.
//!
//! MUST NEVER: resolve a name, open a connection, widen a range (the two
//! private IPv4 ranges below are exact: `172.16.0.0/12` is `172.16` to
//! `172.31`, and `172.15` / `172.32` are remote), echo the input in an error
//! (it may carry `user:password@`), or accept userinfo.

use core::net::{Ipv4Addr, Ipv6Addr};
use core::str::FromStr;
use std::fmt;

/// Where the endpoint is, as far as its text can tell.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostClass {
    /// `localhost`, `127.0.0.0/8`, `::1`, `::ffff:127.0.0.0/104`.
    SameMachine,
    /// `10/8`, `172.16/12`, `192.168/16`, link-local `169.254/16` and
    /// `fe80::/10`, and IPv6 unique-local `fc00::/7` (a labelled judgement:
    /// the IPv6 counterpart of RFC 1918, which the ruling's list did not name).
    LocalNetwork,
    /// A public IP literal, or a dotted name outside the private suffixes.
    Remote,
    /// A name whose location cannot be read from its text (`myhost.local`,
    /// `host.docker.internal`, a single-label name, `0.0.0.0`). Never resolved.
    Unknown,
}

impl HostClass {
    /// The label the UI's "detected as" line shows.
    pub const fn label(self) -> &'static str {
        match self {
            HostClass::SameMachine => "same machine",
            HostClass::LocalNetwork => "local network",
            HostClass::Remote => "remote",
            HostClass::Unknown => "unknown",
        }
    }

    /// A stable id for the bridge and for tests (never shown).
    pub const fn id(self) -> &'static str {
        match self {
            HostClass::SameMachine => "same_machine",
            HostClass::LocalNetwork => "local_network",
            HostClass::Remote => "remote",
            HostClass::Unknown => "unknown",
        }
    }
}

/// A successfully classified host.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostInfo {
    /// The class.
    pub class: HostClass,
    /// The host, lower-cased, without brackets or a trailing dot. For display
    /// only; it never carries userinfo (that is refused).
    pub host: String,
    /// The port if the input stated one. **Absent means absent**, never a
    /// default port such as 80 or 443.
    pub port: Option<u16>,
}

/// Why an input was refused. Static strings only: an error must never echo the
/// input, which may hold `user:password@host`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostError {
    /// Nothing to classify.
    Empty,
    /// `user:pw@host`: credentials in an address are refused outright.
    Userinfo,
    /// A scheme other than `http` or `https`.
    Scheme,
    /// Whitespace, a control character, a backslash or a percent sign. The
    /// last two are how parsers disagree about where the host ends, and a
    /// zone id (`%eth0`) names an interface, not a place.
    BadCharacter,
    /// Brackets that do not enclose exactly one IPv6 address, or junk after `]`.
    BadBrackets,
    /// A port that is empty, non-numeric, zero or above 65535.
    BadPort,
    /// A host that is not a valid name or address.
    BadHost,
    /// A numeric-looking host that is not a strict dotted-quad (`127.1`,
    /// `2130706433`, `0x7f.0.0.1`, `010.0.0.1`). Many stacks read these as an
    /// address, others as a name; classifying either way would be a guess.
    AmbiguousNumeric,
}

impl HostError {
    /// The reason in words, for the settings screen.
    pub const fn reason(self) -> &'static str {
        match self {
            HostError::Empty => "the address is empty",
            HostError::Userinfo => "a user name or password in the address is not allowed",
            HostError::Scheme => "only http:// and https:// addresses are supported",
            HostError::BadCharacter => "the address has a space, control character, backslash or percent sign",
            HostError::BadBrackets => "square brackets must enclose exactly one IPv6 address",
            HostError::BadPort => "the port must be a number from 1 to 65535",
            HostError::BadHost => "the host is not a valid name or address",
            HostError::AmbiguousNumeric => {
                "a number-like host such as 127.1 is ambiguous; write the full dotted address, for example 127.0.0.1"
            }
        }
    }
}

impl fmt::Display for HostError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.reason())
    }
}

impl std::error::Error for HostError {}

/// Name suffixes that name a private network, so a dotted name ending in one
/// is [`HostClass::Unknown`] and not [`HostClass::Remote`]. Labelled judgement
/// from the IANA special-use list (RFC 6761 `.localhost`, RFC 8375
/// `home.arpa`) and the common conventions (`.local` mDNS, `.internal`
/// Docker/Kubernetes/Lima, `.lan`, `.home`, `.localdomain`, `.corp`,
/// `.intranet`). Anything else with a dot is treated as a public DNS name.
const PRIVATE_SUFFIXES: &[&str] =
    &["local", "localhost", "internal", "lan", "home", "localdomain", "corp", "intranet", "home.arpa"];

/// Classify the host part of `input`.
///
/// `input` may be a bare host (`localhost`, `192.168.1.9`, `::1`), a host with
/// a port (`localhost:11434`, `[::1]:8080`), or a full `http(s)://` base URL
/// (anything after the authority is ignored). Userinfo, other schemes,
/// ambiguous numeric hosts, bad ports and malformed brackets are refused.
pub fn classify_host(input: &str) -> Result<HostInfo, HostError> {
    let text = input.trim();
    if text.is_empty() {
        return Err(HostError::Empty);
    }
    if text.bytes().any(|b| b <= 0x20 || b == 0x7F || b == b'\\' || b == b'%') || !text.is_ascii() {
        return Err(HostError::BadCharacter);
    }
    let rest = match text.find("://") {
        Some(i) => {
            let scheme = &text[..i];
            if !scheme.eq_ignore_ascii_case("http") && !scheme.eq_ignore_ascii_case("https") {
                return Err(HostError::Scheme);
            }
            &text[i + 3..]
        }
        None => text,
    };
    let authority = rest.split(['/', '?', '#']).next().unwrap_or("");
    if authority.contains('@') {
        return Err(HostError::Userinfo);
    }
    if authority.is_empty() {
        return Err(HostError::Empty);
    }

    let (host, port) = split_host_port(authority)?;
    let class = classify_name_or_ip(&host)?;
    Ok(HostInfo { class, host, port })
}

/// Split an authority into a normalised host and an optional port.
fn split_host_port(authority: &str) -> Result<(String, Option<u16>), HostError> {
    if let Some(after) = authority.strip_prefix('[') {
        let close = after.find(']').ok_or(HostError::BadBrackets)?;
        let inner = &after[..close];
        let tail = &after[close + 1..];
        if inner.is_empty() || !inner.contains(':') || inner.contains('[') {
            return Err(HostError::BadBrackets);
        }
        let port = match tail {
            "" => None,
            t => Some(parse_port(t.strip_prefix(':').ok_or(HostError::BadBrackets)?)?),
        };
        return Ok((inner.to_ascii_lowercase(), port));
    }
    if authority.contains(']') || authority.contains('[') {
        return Err(HostError::BadBrackets);
    }
    // More than one colon and no brackets: only a bare IPv6 literal is valid
    // (it cannot carry a port, which is why URLs bracket it).
    if authority.matches(':').count() > 1 {
        return match Ipv6Addr::from_str(authority) {
            Ok(_) => Ok((authority.to_ascii_lowercase(), None)),
            Err(_) => Err(HostError::BadHost),
        };
    }
    match authority.split_once(':') {
        Some((h, p)) => Ok((h.to_ascii_lowercase(), Some(parse_port(p)?))),
        None => Ok((authority.to_ascii_lowercase(), None)),
    }
}

/// Parse a port: digits only, 1 to 65535. Zero is refused because it is the
/// "no port" sentinel elsewhere and is never a real listening port.
fn parse_port(text: &str) -> Result<u16, HostError> {
    if text.is_empty() || text.len() > 5 || !text.bytes().all(|b| b.is_ascii_digit()) {
        return Err(HostError::BadPort);
    }
    match text.parse::<u16>() {
        Ok(0) | Err(_) => Err(HostError::BadPort),
        Ok(n) => Ok(n),
    }
}

/// Classify an already-split, lower-cased host (a name or an IP literal).
fn classify_name_or_ip(host: &str) -> Result<HostClass, HostError> {
    if host.contains(':') {
        let v6 = Ipv6Addr::from_str(host).map_err(|_| HostError::BadHost)?;
        return Ok(classify_v6(v6));
    }
    let name = host.strip_suffix('.').unwrap_or(host);
    if name.is_empty() {
        return Err(HostError::BadHost);
    }
    // Number-like: every label is made of digits (and, for hex forms, `x` and
    // hex digits) and the first starts with a digit. A strict dotted-quad is
    // an address; anything else of this shape is ambiguous, not a name.
    if name.as_bytes()[0].is_ascii_digit() && name.bytes().all(|b| b.is_ascii_hexdigit() || b == b'.' || b == b'x') {
        return match Ipv4Addr::from_str(name) {
            Ok(v4) => Ok(classify_v4(v4)),
            Err(_) => Err(HostError::AmbiguousNumeric),
        };
    }
    if !is_valid_name(name) {
        return Err(HostError::BadHost);
    }
    if name == "localhost" {
        return Ok(HostClass::SameMachine);
    }
    let dotted = name.contains('.');
    let private = PRIVATE_SUFFIXES
        .iter()
        .any(|s| name == *s || name.strip_suffix(*s).is_some_and(|p| p.ends_with('.')));
    Ok(if dotted && !private { HostClass::Remote } else { HostClass::Unknown })
}

/// RFC 1123 labels plus `_` (Docker Compose service names use it): 1 to 63
/// characters of `a-z 0-9 - _`, not starting or ending with `-`, 253 in all.
fn is_valid_name(name: &str) -> bool {
    name.len() <= 253
        && name.split('.').all(|l| {
            !l.is_empty()
                && l.len() <= 63
                && !l.starts_with('-')
                && !l.ends_with('-')
                && l.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-' || b == b'_')
        })
}

/// IPv4 classes. Written octet by octet, not with `Ipv4Addr::is_private`, so
/// the boundaries in the ruling are visible and a widened range is a diff.
fn classify_v4(a: Ipv4Addr) -> HostClass {
    let [o0, o1, _, _] = a.octets();
    match (o0, o1) {
        (127, _) => HostClass::SameMachine,
        (10, _) => HostClass::LocalNetwork,
        (172, 16..=31) => HostClass::LocalNetwork,
        (192, 168) => HostClass::LocalNetwork,
        (169, 254) => HostClass::LocalNetwork,
        // "This host" (0.0.0.0) and the rest of 0/8 connect to the local
        // machine on some stacks and are invalid on others: not guessed at.
        (0, _) => HostClass::Unknown,
        _ => HostClass::Remote,
    }
}

/// IPv6 classes; an IPv4-mapped address is classified by the IPv4 it carries.
fn classify_v6(a: Ipv6Addr) -> HostClass {
    if a == Ipv6Addr::LOCALHOST {
        return HostClass::SameMachine;
    }
    if a == Ipv6Addr::UNSPECIFIED {
        return HostClass::Unknown;
    }
    let s = a.segments();
    if s[..5] == [0, 0, 0, 0, 0] && s[5] == 0xffff {
        let v4 = Ipv4Addr::new((s[6] >> 8) as u8, s[6] as u8, (s[7] >> 8) as u8, s[7] as u8);
        return classify_v4(v4);
    }
    if s[0] & 0xffc0 == 0xfe80 || s[0] & 0xfe00 == 0xfc00 {
        return HostClass::LocalNetwork;
    }
    HostClass::Remote
}

#[cfg(test)]
mod tests {
    use super::*;

    fn class(s: &str) -> HostClass {
        classify_host(s).unwrap_or_else(|e| panic!("{s:?}: {e}")).class
    }

    /// Protects: the exact table in the LLM-2 bar (same machine).
    #[test]
    fn same_machine_table() {
        for s in ["localhost", "127.0.0.1", "127.5.5.5", "::1", "[::1]:8080", "::ffff:127.0.0.1"] {
            assert_eq!(class(s), HostClass::SameMachine, "{s}");
        }
    }

    /// Protects: the LAN table, including both ends of 172.16/12 (the
    /// `172.31.255.255` row is the one a widened range would still pass, so
    /// the remote table below carries the mutation catcher).
    #[test]
    fn local_network_table() {
        for s in ["10.0.0.5", "172.16.0.1", "172.31.255.255", "192.168.1.9", "169.254.1.1", "fe80::1"] {
            assert_eq!(class(s), HostClass::LocalNetwork, "{s}");
        }
    }

    /// Protects: `172.15` and `172.32` sit on either side of 172.16/12 and must
    /// stay remote. MUTATION: widen the arm to `(172, _)` and `172.15.0.1`
    /// turns `LocalNetwork`: this test goes red.
    #[test]
    fn remote_table_and_172_boundaries() {
        for s in ["172.15.0.1", "172.32.0.1", "8.8.8.8", "api.openai.com", "11.0.0.1", "192.169.0.1", "169.255.0.1"] {
            assert_eq!(class(s), HostClass::Remote, "{s}");
        }
        assert_eq!(class("[2001:db8::1]"), HostClass::Remote);
        assert_eq!(class("2606:4700::1111"), HostClass::Remote);
    }

    /// Protects: names are never resolved, so these are Unknown.
    #[test]
    fn unknown_names_are_never_resolved() {
        for s in ["host.docker.internal", "myhost.local", "myserver", "0.0.0.0", "::", "box.home.arpa", "nas.lan"] {
            assert_eq!(class(s), HostClass::Unknown, "{s}");
        }
        // A name that merely ends in a private word is still a public name.
        assert_eq!(class("evil-local.com"), HostClass::Remote);
        assert_eq!(class("notlocalhost.com"), HostClass::Remote);
        assert_eq!(class("x.localhost"), HostClass::Unknown);
    }

    /// Protects: ports and brackets parse, and an absent port stays absent.
    #[test]
    fn ports_and_brackets_parse() {
        let h = classify_host("[::1]:8080").unwrap();
        assert_eq!((h.host.as_str(), h.port), ("::1", Some(8080)));
        let h = classify_host("localhost:11434").unwrap();
        assert_eq!((h.host.as_str(), h.port), ("localhost", Some(11434)));
        assert_eq!(classify_host("localhost").unwrap().port, None);
        assert_eq!(classify_host("::1").unwrap().port, None);
        assert_eq!(classify_host("[::1]").unwrap().port, None);
        assert_eq!(classify_host("LOCALHOST.").unwrap().class, HostClass::SameMachine);
        assert_eq!(classify_host("http://192.168.1.9:1234/v1").unwrap().port, Some(1234));
        assert_eq!(classify_host("https://API.OpenAI.com/v1").unwrap().host, "api.openai.com");
        assert_eq!(classify_host(" localhost:1 ").unwrap().port, Some(1));
        assert_eq!(classify_host("localhost:65535").unwrap().port, Some(65535));
    }

    /// Protects: userinfo is refused in every spelling, and the error never
    /// echoes the input. MUTATION: delete the `contains('@')` check and the
    /// first loop goes red.
    #[test]
    fn userinfo_is_refused_and_never_echoed() {
        for s in ["user:pw@host", "user@localhost", "http://user:pw@localhost:11434/v1", "https://:pw@api.openai.com", "a@b@c"] {
            assert_eq!(classify_host(s), Err(HostError::Userinfo), "{s}");
        }
        let e = classify_host("user:hunter2@host").unwrap_err();
        assert!(!format!("{e} {e:?}").contains("hunter2"));
        // '@' after the authority is path text, not userinfo.
        assert_eq!(class("http://localhost/a@b"), HostClass::SameMachine);
    }

    /// Protects: every refusal reason, one input each.
    #[test]
    fn refusals_table() {
        let cases: [(&str, HostError); 22] = [
            ("", HostError::Empty),
            ("   ", HostError::Empty),
            ("http://", HostError::Empty),
            ("ftp://localhost", HostError::Scheme),
            ("file:///etc", HostError::Scheme),
            ("local host", HostError::BadCharacter),
            ("local\thost", HostError::BadCharacter),
            ("http://localhost\\@evil.com", HostError::BadCharacter),
            ("fe80::1%eth0", HostError::BadCharacter),
            ("[::1", HostError::BadBrackets),
            ("[]:80", HostError::BadBrackets),
            ("[localhost]:80", HostError::BadBrackets),
            ("[::1]x", HostError::BadBrackets),
            ("[::1]:", HostError::BadPort),
            ("localhost:", HostError::BadPort),
            ("localhost:0", HostError::BadPort),
            ("localhost:65536", HostError::BadPort),
            ("localhost:99999999", HostError::BadPort),
            ("localhost:-1", HostError::BadPort),
            ("localhost:8o", HostError::BadPort),
            ("1:2:3:4:5:6:7:8:9", HostError::BadHost),
            ("-bad.example.com", HostError::BadHost),
        ];
        for (s, want) in cases {
            assert_eq!(classify_host(s), Err(want), "{s:?}");
        }
    }

    /// Protects: number-like hosts that are not a strict dotted quad are
    /// refused rather than guessed (they read as loopback on some stacks).
    #[test]
    fn ambiguous_numeric_hosts_are_refused() {
        for s in ["127.1", "2130706433", "0x7f.0.0.1", "010.0.0.1", "127.0.0.256", "1.2.3", "1.2.3.4.5", "0177.0.0.1"] {
            assert_eq!(classify_host(s), Err(HostError::AmbiguousNumeric), "{s}");
        }
        // A real name that starts with a digit is still a name.
        assert_eq!(class("3dprint.example.com"), HostClass::Remote);
    }

    /// Protects: IPv4-mapped IPv6 follows the IPv4 it carries, in both
    /// directions (a mapped LAN address is LAN, a mapped public one remote).
    #[test]
    fn ipv4_mapped_follows_the_embedded_address() {
        assert_eq!(class("::ffff:10.1.2.3"), HostClass::LocalNetwork);
        assert_eq!(class("::ffff:8.8.8.8"), HostClass::Remote);
        assert_eq!(class("[::ffff:127.9.9.9]:1"), HostClass::SameMachine);
        assert_eq!(class("fd12:3456::1"), HostClass::LocalNetwork);
        assert_eq!(class("febf::1"), HostClass::LocalNetwork);
        assert_eq!(class("fec0::1"), HostClass::Remote, "site-local fec0::/10 is deprecated, not link-local");
    }

    /// Protects: the classes' words, which the UI shows verbatim.
    #[test]
    fn labels_are_the_ruling_words() {
        assert_eq!(HostClass::SameMachine.label(), "same machine");
        assert_eq!(HostClass::LocalNetwork.label(), "local network");
        assert_eq!(HostClass::Remote.label(), "remote");
        assert_eq!(HostClass::Unknown.label(), "unknown");
    }
}
