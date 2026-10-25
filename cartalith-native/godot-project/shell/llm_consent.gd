extends RefCounted
class_name LlmConsent

## **The consent screen** (`LLM_ASSISTANT_SCOPE.md` s.1 rule 6, s.4 rule 6; Ruling DD
## items 3 and 5; LLM-2 bar: "cannot be skipped and appears before the first call to each
## new endpoint").
##
## One modal card in the shell's modal vocabulary (`DccWidgets.modal_card`), laid out as
## the owner-approved LLM-0 mock draws it (`design/llm-assistant-2026-10-25/index.html`,
## `consentH()`): what is sent, to which endpoint, what is never sent, that **no spend
## limit is enforced**, a *link* to the provider's terms, and that output is data.
##
## ## What "cannot be skipped" means here
##
## Two things, both enforced rather than promised:
##
## * The only way to record consent is the card's "I understand, enable" button
##   (`on_agree`). Esc, the close box, and "Not now" all take the decline path and record
##   nothing -- `DccWidgets.modal_choices` routes `canceled`, `close_requested` and the
##   header close box to the same dismissal.
## * The gate (`cartalith-llm::settings::authorize`) refuses every request to an origin
##   that is not in `DccSettings.llm_consented_origins()` with `consent_needed`. No code
##   path sends around it, and LLM-3's only door is `run_authorized_chat`.
##
## ## The text must be true (scope s.4 rule 6)
##
## Every sentence here is a statement about behaviour that exists or a policy the gate
## enforces; a line about commands appears only while the Commands switch is on, and the
## context line lists only the sources the user ticked. The terms are **linked, not
## paraphrased**: a `LinkButton` whose `uri` is the provider's page (`cartalith-llm`'s
## `terms_url`, which has no entry for a local server, so none is drawn).
##
## MUST NEVER: record consent on any path but the agree button; show a link the table
## does not hold; name an endpoint other than the origin the gate will check (the caller
## passes `WorldGen.llm_endpoint_info`'s `origin`).

## The card's width in px. `DccWidgets.MODAL_W` (340) is the stock card; the consent text is long
## enough that 380 reads as prose rather than a column. Labelled judgement. Every wrapping label
## below is given the matching minimum WIDTH (`_wrap`): a `wrap_controls` dialog measures an
## autowrapped label at width 0 on its first layout and sizes the window to the resulting tower
## (measured 8 426 px tall before this), so the wrap width must be stated, not inferred.
const CARD_W := 380
## The card body's horizontal padding on each side (`DccWidgets.modal_card`'s `bar_pad_x`).
const _BODY_PAD := 14
## The inset's horizontal content margins, summed (`DccWidgets.modal_inset`: 11 each side).
const _INSET_PAD := 22

## Opens the consent card for `info` (an `llm_endpoint_info` dictionary with `ok` true)
## and returns `{"card", "choices", "terms"}` so a probe can reach the buttons.
## `preset` is the picker row (for the terms link) or `{}`. `on_agree` runs only after
## the user presses "I understand, enable"; `on_decline` runs on every other way out.
static func present(host, info: Dictionary, preset: Dictionary, on_agree: Callable,
		on_decline: Callable = Callable()) -> Dictionary:
	var origin := String(info.get("origin", ""))
	var class_label := String(info.get("class_label", "unknown"))
	var card := DccWidgets.modal_card(host, "Before the first request", DccWidgets.MODAL_CONFIRM, CARD_W)
	var body: VBoxContainer = card["body"]

	_wrap(DccWidgets.modal_prose(body, "Turning this on sends text to %s (%s). Each request sends:" % [origin, class_label]), 0)
	var sent := DccWidgets.modal_inset(body)
	_line(sent, "the text you type")
	_line(sent, "the context sources you ticked, and no others: %s" % _context_sources())
	if bool(DccSettings.llm_value("commands")):
		_line(sent, "the list of commands the assistant may propose")
		if bool(DccSettings.llm_value("ctx_params")):
			_line(sent, "for a setting proposal: the names, ranges, descriptions and current values of the setting groups the model asks for")

	var never := DccTheme.label(
		"Never sent: your whole Vault, the save file, file paths, the machine name. Your key goes only in the provider request header.",
		"text_secondary", DccTheme.FS_SMALL)
	body.add_child(_wrap(never, 0))

	var spend := DccTheme.label(
		"No spend limit is enforced. The token counter shows usage; a hosted provider bills you directly. Cartalith does not control what the server does with your text.",
		"text_secondary", DccTheme.FS_SMALL)
	body.add_child(_wrap(spend, 0))

	var terms: LinkButton = null
	var url := String(preset.get("terms_url", ""))
	if url != "":
		terms = LinkButton.new()
		terms.text = "Provider terms (linked, not paraphrased)"
		terms.uri = url
		terms.tooltip_text = url
		terms.underline = LinkButton.UNDERLINE_MODE_ALWAYS
		terms.focus_mode = Control.FOCUS_NONE
		terms.add_theme_font_size_override("font_size", DccTheme.FS_SMALL)
		terms.add_theme_color_override("font_color", DccTheme.c("accent"))
		## A phone finger needs a real target, not a one-line text run.
		if DccTheme.is_phone():
			terms.custom_minimum_size.y = DccTheme.PHONE_TAP_MIN
		body.add_child(terms)

	var data_note := DccTheme.label(
		"Output is marked model-generated and is data: nothing runs without your Apply. Worlds generate identically with this off.",
		"text_ghost", DccTheme.FS_MICRO)
	body.add_child(_wrap(data_note, 0))

	## Consent is recorded HERE and only here, inside the agree answer. Every other way out
	## (Not now, Esc, the close box) reaches `decline` and writes nothing.
	var agree := func() -> void:
		DccSettings.llm_add_consent(origin)
		if on_agree.is_valid():
			on_agree.call()
	var decline := func() -> void:
		if on_decline.is_valid():
			on_decline.call()
	var choices := DccWidgets.modal_choices(card, {
		"cancel": "Not now",
		"safe": {"text": "I understand, enable", "on": agree},
		"on_cancel": decline,
	})
	DccWidgets.modal_present(card, host)
	## **Re-fit after the first layout (pointer and tablet).** A `wrap_controls` dialog sizes itself
	## from its content's minimum on the first pop-up, when the wrapped labels have not yet been
	## given their width, and it only ever grows: measured here, a card whose content is 355 px tall
	## came up 1 799 px tall, a dark column the height of the screen (`_dbg` of 2026-10-25). One
	## frame later the content is right; `reset_size()` takes the window back to it, and
	## `move_to_center()` undoes the centring the oversized first size caused. A phone card is
	## full-screen by `phone_present`, so it is left alone.
	if not DccTheme.is_phone():
		var dlg: Window = card["dialog"]
		var refit := func() -> void:
			await host.get_tree().process_frame
			if is_instance_valid(dlg) and dlg.visible:
				dlg.reset_size()
				dlg.move_to_center()
		refit.call_deferred()
	return {"card": card, "choices": choices, "terms": terms}

## One bullet row inside the inset.
static func _line(parent: Control, text: String) -> void:
	var l := DccTheme.label("•  " + text, "text", DccTheme.FS_BODY)
	parent.add_child(_wrap(l, _INSET_PAD))

## Makes `l` wrap at the card's real width (`CARD_W` less the body padding and `extra`): autowrap
## plus an explicit minimum width. Returns `l`.
static func _wrap(l: Label, extra: int) -> Label:
	l.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	l.custom_minimum_size.x = CARD_W - 2 * _BODY_PAD - extra
	return l

## The context sources the user ticked, in words; "none" when none (the default).
static func _context_sources() -> String:
	var on: Array[String] = []
	if bool(DccSettings.llm_value("ctx_entity")):
		on.append("current entity")
	if bool(DccSettings.llm_value("ctx_selection")):
		on.append("selection")
	if bool(DccSettings.llm_value("ctx_notes")):
		on.append("named Vault notes")
	if bool(DccSettings.llm_value("ctx_params")):
		on.append("generation parameters")
	return ", ".join(on) if not on.is_empty() else "none"
