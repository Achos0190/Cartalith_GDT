//! One process, one global subscriber, so one ordered test: off by default, then a span and a
//! bridged wgpu record land in the file with their fields, and a planted note sentence / key-shaped
//! string never does (TOOLING_PLAN.md TR-1, TR-5).

const NOTE: &str = "The queen hides the silver sword beneath the chapel";
const KEY: &str = "sk-CANARY0123456789abcdef";

#[test]
fn off_by_default_then_on_writes_allowlisted_fields_only() {
    // Off by default: no subscriber, a span is disabled, shell hook refuses, no file appears.
    assert!(!cartalith_trace::is_on());
    assert!(!tracing::dispatcher::has_been_set());
    assert!(tracing::info_span!("gen::x").is_disabled());
    assert!(!cartalith_trace::shell_event("menu.run", 1.0, 1));
    assert!(cartalith_trace::start("off", &std::env::temp_dir()).is_none());

    let dir = std::env::temp_dir().join(format!("cartalith_trace_test_{}", std::process::id()));
    let path = cartalith_trace::start("info", &dir).expect("starts");
    assert!(cartalith_trace::is_on());
    {
        let s = tracing::info_span!("gen::erosion", cells = 2048u64 * 1024, note = NOTE, key = KEY, op = KEY);
        let _g = s.enter();
        std::thread::sleep(std::time::Duration::from_millis(3));
        tracing::info!(count = 7u64, text = NOTE, op = NOTE, "free text {}", NOTE);
    }
    assert!(cartalith_trace::shell_event("menu.run", 2.5, 3));
    assert!(!cartalith_trace::shell_event(NOTE, 0.0, 0));
    cartalith_trace::forward_log(
        &log::Record::builder().target("wgpu_hal::vulkan").level(log::Level::Info).args(format_args!("adapter: Fake GPU")).build(),
    );
    cartalith_trace::forward_log(
        &log::Record::builder().target("cartalith_godot").level(log::Level::Info).args(format_args!("{NOTE}")).build(),
    );
    cartalith_trace::flush();
    let text = std::fs::read_to_string(&path).unwrap();

    // Canary: neither string reached the file, in any field or message.
    assert!(!text.contains("queen") && !text.contains("sword") && !text.contains("sk-CANARY"), "{text}");
    // The span is there with its allowlisted number and a real duration.
    let span = text.lines().find(|l| l.contains("\"name\":\"gen::erosion\"")).expect("span line");
    assert!(span.contains("\"cells\":2097152"), "{span}");
    let us: u64 = span.split("\"elapsed_us\":").nth(1).unwrap().split(',').next().unwrap().parse().unwrap();
    assert!(us >= 3000, "{span}");
    assert!(span.contains("\"t\":\"span\""));
    // The shell op and the bridged wgpu record are there.
    assert!(text.contains("\"op\":\"menu.run\""), "{text}");
    assert!(text.contains("adapter: Fake GPU"), "{text}");
    // Dropped fields are only counted.
    assert!(text.lines().any(|l| l.contains("\"count\":7") && l.contains("\"dropped\":")), "{text}");

    cartalith_trace::stop();
    assert!(!cartalith_trace::is_on());
    tracing::info!(count = 99u64);
    cartalith_trace::flush();
    assert!(!std::fs::read_to_string(&path).unwrap().contains("\"count\":99"));
    let _ = std::fs::remove_dir_all(&dir);
}
