#![allow(clippy::excessive_precision)]
//! Golden-parity tests for cartalith-civ's water-body classification
//! (`buildWaterBodies`, reference HTML line 5753) against the real
//! reference engine. `PHASE2_SCOPE.md` milestone 2. Generated from a Node
//! `vm` extraction run (harness itself is transient, not checked in) --
//! same technique/discipline as every other golden test in this workspace.
//! Cases reuse `cartalith-engine/tests/golden_parity_carve.rs`'s exact
//! fixture configs (gw/gh/seed/world/w_iters=12), and `field[0..5]` from
//! this independent extraction matched that fixture's own `expected_field`
//! exactly, cross-checking the harness is faithful.
//!
//! One real harness bug found and root-caused during extraction, not
//! papered over: the reference's own `state` literal defaults
//! `tect.seed` to `(Math.random()*99999)|0` at script-load time (line
//! 2264) -- the real per-generation seed lives at `state.tect.seed`, not
//! a top-level `state.seed`. Setting the wrong field left the harness
//! silently running with a fresh random seed every process invocation,
//! producing genuinely nondeterministic output across runs despite
//! identical harness code -- confirmed by running the extraction twice
//! and diffing. Fixed by setting `state.tect.seed` (matching
//! `WorldParams.tect.seed` on the Rust side); re-verified deterministic
//! across two separate process runs before trusting the extracted data.
//!
//! Classification is categorical (`Uint8`, 0 land / 1 ocean / 2 lake) --
//! bit-exact match required, any mismatch is a real bug. `fill_level` is
//! a continuous `f32` field; `1e-4` absolute+relative tolerance, matching
//! this workspace's convention for fields downstream of
//! `Math.hypot`/priority-flood-derived arithmetic (not required to be
//! bit-identical across JS/Rust).
//!
//! **`case_1_world_wrap`'s `expected_classification`/`expected_fill` were
//! re-baselined 2026-09-21, `LARGE_ITEM_RULINGS.md`'s Ruling Q.**
//! `cartalith_civ::build_water_bodies` no longer matches the reference's
//! own `buildWaterBodies` on this one output -- it is a DELIBERATE
//! divergence (`HYDROLOGY_CLASSIFICATION_RESEARCH.md`), not a parity port,
//! for the ocean/lake classification only. `case_0_region` is untouched
//! (its below-sea component was already the boundary-touching one, so old
//! and new rules agree there -- re-run and confirmed byte/tolerance
//! identical against the values already in this file). `case_1_world_wrap`
//! moved because two below-sea cells at row 0 (a pole, always a real
//! boundary under this crate's own `y==0`/`y==gh-1` edge test, wrapped or
//! not) were the smaller of two components and lost the old largest-wins
//! rule; under Ruling Q they touch the map's real boundary and become
//! ocean regardless of size. `classification[0]`/`classification[1]`
//! moved lake(2) -> ocean(1), which changes the seed set the priority-flood
//! depression pass fills from (`build_water_bodies` seeds it from every
//! `out[i]==1` cell), so `fill_level` moved on cells whose fill surface is
//! now reached from the newly-ocean seed instead of the old one -- a real,
//! understood second-order effect of the reclassification, not a separate
//! bug. Both arrays below are `cartalith_civ::build_water_bodies`'s own
//! actual output on this fixture, captured 2026-09-21 after the change
//! (`cargo run -p cartalith-civ --example _dump_waterbodies`, deleted
//! after use).

fn assert_close(actual: &[f32], expected: &[f32], label: &str) {
    const ATOL: f32 = 1e-4;
    const RTOL: f32 = 1e-4;
    assert_eq!(actual.len(), expected.len(), "{label}: length mismatch");
    for (i, (&a, &e)) in actual.iter().zip(expected.iter()).enumerate() {
        let tol = ATOL + RTOL * e.abs();
        assert!(
            (a - e).abs() <= tol,
            "{label} index {i}: got {a}, expected {e} (diff {}, tol {tol})",
            (a - e).abs()
        );
    }
}

#[test]
fn water_bodies_case_0_region() {
    // case 0: region: gw=14 gh=11 seed=24601 world=false. No above-sea
    // pooled lakes form in this configuration -- classification is 0/1
    // only (79 land, 75 ocean), which is itself a real, useful case: the
    // priority-flood machinery runs and correctly finds nothing to pool.
    let expected_classification: Vec<u8> = vec![
        0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 0, 0,
        0, 0, 0, 0, 1, 1, 1, 0, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0, 1, 1, 0, 1, 1, 1, 1, 1, 0, 0, 0, 0,
        0, 0, 1, 1, 0, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 1, 1, 0, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 1,
        1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    ];
    let expected_fill: Vec<f32> = vec![
        0.8640562295913696f32, 0.7786418199539185f32, 0.6850417256355286f32, 0.6560115814208984f32, 0.6181289553642273f32, 0.4768886864185333f32,
        0.4512621760368347f32, 0.4579582214355469f32, 0.455786794424057f32, 0.38202551007270813f32, 0.34962502121925354f32, 0.33881452679634094f32,
        0.3697913885116577f32, 0.3857390284538269f32, 0.850577175617218f32, 0.7163345813751221f32, 0.6448685526847839f32, 0.6420062184333801f32,
        0.6229793429374695f32, 0.5320773124694824f32, 0.45887455344200134f32, 0.48031485080718994f32, 0.44087833166122437f32, 0.3887729048728943f32,
        0.28496071696281433f32, 0.3004293441772461f32, 0.3132499158382416f32, 0.4020007848739624f32, 0.8342151641845703f32, 0.7243708968162537f32,
        0.6292590498924255f32, 0.5595474243164062f32, 0.5797417163848877f32, 0.5093994736671448f32, 0.45702531933784485f32, 0.5626664757728577f32,
        0.4110821783542633f32, 0.38582703471183777f32, 0.29495370388031006f32, 0.381040096282959f32, 0.4055592119693756f32, 0.3254455626010895f32,
        0.765463650226593f32, 0.7091397643089294f32, 0.5621345043182373f32, 0.5395821332931519f32, 0.49799540638923645f32, 0.45989954471588135f32,
        0.3377099931240082f32, 0.3753055930137634f32, 0.30223149061203003f32, 0.5471212863922119f32, 0.3423593044281006f32, 0.42524585127830505f32,
        0.570194661617279f32, 0.35602033138275146f32, 0.6862545609474182f32, 0.7032239437103271f32, 0.6526187062263489f32, 0.5404073596000671f32,
        0.48919087648391724f32, 0.4559832513332367f32, 0.3549758195877075f32, 0.37110722064971924f32, 0.22203518450260162f32, 0.3894047141075134f32,
        0.4068662226200104f32, 0.48743554949760437f32, 0.526330292224884f32, 0.4176136553287506f32, 0.6445929408073425f32, 0.6325864195823669f32,
        0.6543266773223877f32, 0.6027469635009766f32, 0.5864615440368652f32, 0.5073954463005066f32, 0.35393452644348145f32, 0.384827584028244f32,
        0.42050430178642273f32, 0.20669172704219818f32, 0.22476257383823395f32, 0.16364707052707672f32, 0.23192164301872253f32, 0.2639058232307434f32,
        0.6234947443008423f32, 0.6234957575798035f32, 0.6263570189476013f32, 0.6071434617042542f32, 0.6741071343421936f32, 0.6340454816818237f32,
        0.3279743492603302f32, 0.3781207203865051f32, 0.4437485337257385f32, 0.2995627820491791f32, 0.13135510683059692f32, 0.08891052007675171f32,
        0.1343614161014557f32, 0.24156707525253296f32, 0.6558512449264526f32, 0.630266547203064f32, 0.604451596736908f32, 0.565191924571991f32,
        0.5171617865562439f32, 0.5324947237968445f32, 0.5107908248901367f32, 0.3672998547554016f32, 0.3028997778892517f32, 0.21875089406967163f32,
        0.157021164894104f32, 0.1164918914437294f32, 0.1514834314584732f32, 0.1154087707400322f32, 0.6755391955375671f32, 0.6757492423057556f32,
        0.6273571848869324f32, 0.5134481191635132f32, 0.4704248607158661f32, 0.346716970205307f32, 0.3457725942134857f32, 0.4519369602203369f32,
        0.23998835682868958f32, 0.2084149420261383f32, 0.10973665863275528f32, 0.18431293964385986f32, 0.1415526270866394f32, 0.005264172330498695f32,
        0.6933126449584961f32, 0.681037187576294f32, 0.621044933795929f32, 0.533515453338623f32, 0.43472614884376526f32, 0.39808589220046997f32,
        0.339803010225296f32, 0.28785380721092224f32, 0.21926209330558777f32, 0.10646077990531921f32, 0.12476237863302231f32, 0.21373078227043152f32,
        0.12487001717090607f32, 0.005248506087809801f32, 0.7472889423370361f32, 0.6620296239852905f32, 0.5572002530097961f32, 0.5241872668266296f32,
        0.38885411620140076f32, 0.37194257974624634f32, 0.3441046476364136f32, 0.2798146903514862f32, 0.33529844880104065f32, 0.20280706882476807f32,
        0.2382887601852417f32, 0.18706771731376648f32, 0.12252187728881836f32, 0.005232839845120907f32,
    ];

    let mut p = cartalith_engine::WorldParams::defaults(14, 11, 24601);
    p.world = false;
    p.climate.w_iters = 12;
    let ws = cartalith_engine::generate_terrain(&p);
    assert!((ws.sea_level - 0.42f64).abs() < 1e-9, "sea_level mismatch, harness assumption broken");

    let wb = cartalith_civ::build_water_bodies(&ws.field, 14, 11, ws.sea_level, false, Some(&ws.rainfall));

    assert_eq!(wb.classification, expected_classification, "classification");
    assert_close(&wb.fill_level, &expected_fill, "fill_level");
}

#[test]
fn water_bodies_case_1_world_wrap() {
    // case 1: world_wrap: gw=16 gh=12 seed=314159 world=true. RE-BASELINED
    // 2026-09-21 (Ruling Q, see this file's own header) -- now 116 land,
    // 14 ocean, 62 lake (was 127/13/52 under the old size-primary rule),
    // including the x-wrap connected-components/priority-flood path.
    let expected_classification: Vec<u8> = vec![
        1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 2, 2, 0, 0, 2, 0, 0, 0, 0, 0, 0, 2, 0, 0, 2, 2, 2, 2, 0, 0, 0, 2, 0,
        0, 0, 0, 2, 2, 0, 2, 2, 2, 2, 2, 0, 0, 0, 0, 0, 0, 0, 2, 2, 0, 0, 2, 2, 2, 2, 2, 0, 0, 2, 2, 0, 0, 0, 0, 0, 0, 0, 2, 2, 2, 0, 2, 2,
        0, 2, 0, 0, 0, 2, 0, 0, 2, 0, 0, 2, 0, 2, 2, 2, 2, 2, 2, 0, 2, 2, 0, 2, 2, 2, 0, 0, 0, 2, 2, 2, 2, 2, 2, 0, 2, 0, 0, 2, 2, 0, 0, 0,
        0, 0, 0, 2, 2, 0, 0, 0, 2, 0, 0, 2, 2, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 2, 2, 0, 0, 0, 0, 1, 1, 0, 1, 1, 1, 0,
        0, 0, 1, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0,
    ];
    let expected_fill: Vec<f32> = vec![
        0.2478043f32, 0.24909122f32, 0.6786974f32, 0.75004524f32, 0.6631427f32, 0.5706903f32,
        0.55409485f32, 0.61845356f32, 0.5798958f32, 0.48864436f32, 0.51604545f32, 0.44300875f32,
        0.42933866f32, 0.4352145f32, 0.57974815f32, 0.51781934f32, 0.46010676f32, 0.19033732f32,
        0.69944614f32, 0.7010609f32, 0.54690963f32, 0.5469086f32, 0.5469076f32, 0.5516437f32,
        0.52267456f32, 0.48864537f32, 0.4927716f32, 0.49068892f32, 0.43347737f32, 0.47847837f32,
        0.64164823f32, 0.57228905f32, 0.46010777f32, 0.6113015f32, 0.56501627f32, 0.54690963f32,
        0.5469086f32, 0.5469076f32, 0.5469066f32, 0.57728183f32, 0.6313944f32, 0.523051f32,
        0.4927726f32, 0.5514879f32, 0.47062564f32, 0.5769707f32, 0.5894634f32, 0.4601088f32,
        0.4601088f32, 0.6398572f32, 0.54690963f32, 0.5469086f32, 0.5469076f32, 0.5469066f32,
        0.5469056f32, 0.5496647f32, 0.61036223f32, 0.6133828f32, 0.6119778f32, 0.56371784f32,
        0.49430215f32, 0.560956f32, 0.4601108f32, 0.4601098f32, 0.73148936f32, 0.7632583f32,
        0.5469086f32, 0.5469076f32, 0.5469066f32, 0.5469056f32, 0.54690456f32, 0.5685357f32,
        0.7842975f32, 0.5469056f32, 0.5469066f32, 0.57253975f32, 0.52425796f32, 0.7213795f32,
        0.8459461f32, 0.7984103f32, 0.68476295f32, 0.6281154f32, 0.54690963f32, 0.5469086f32,
        0.5469076f32, 0.55985534f32, 0.54690355f32, 0.54690254f32, 0.62201923f32, 0.54690456f32,
        0.57825345f32, 0.5487478f32, 0.5108504f32, 0.5101717f32, 0.6556576f32, 0.82293755f32,
        0.46291864f32, 0.61893463f32, 0.5652268f32, 0.54690963f32, 0.7722459f32, 0.54690355f32,
        0.54690254f32, 0.5469015f32, 0.54690254f32, 0.54690355f32, 0.54690456f32, 0.7044184f32,
        0.5101717f32, 0.5101707f32, 0.5101697f32, 0.46291763f32, 0.46291763f32, 0.46291864f32,
        0.74038523f32, 0.67350644f32, 0.75066924f32, 0.54690254f32, 0.5469015f32, 0.5469005f32,
        0.5469015f32, 0.54690254f32, 0.54690355f32, 0.59197176f32, 0.5101727f32, 0.65634066f32,
        0.60395515f32, 0.4629166f32, 0.4629166f32, 0.48458713f32, 0.6400034f32, 0.6128287f32,
        0.466222f32, 0.6164857f32, 0.5748281f32, 0.5468995f32, 0.5469005f32, 0.69730926f32,
        0.8950326f32, 0.8921517f32, 0.51017374f32, 0.61939f32, 0.6365516f32, 0.4629156f32,
        0.4629156f32, 0.6857683f32, 0.46841913f32, 0.42907578f32, 0.46622097f32, 0.4742268f32,
        0.48121342f32, 0.5468985f32, 0.7109946f32, 0.46301368f32, 0.883307f32, 0.9133646f32,
        0.5906895f32, 0.30499464f32, 0.7254239f32, 0.4629146f32, 0.6404506f32, 0.6809148f32,
        0.46151397f32, 0.42907476f32, 0.42907375f32, 0.42907274f32, 0.45892563f32, 0.49252275f32,
        0.6585263f32, 0.41218013f32, 0.28808933f32, 0.81657284f32, 0.3861949f32, 0.28861022f32,
        0.33218986f32, 0.44830695f32, 0.7278715f32, 0.78261775f32, 0.35190308f32, 0.5308783f32,
        0.43067744f32, 0.42027646f32, 0.4476934f32, 0.5603108f32, 0.67092896f32, 0.3109951f32,
        0.25467834f32, 0.3274588f32, 0.39751482f32, 0.7357266f32, 0.5704076f32, 0.52112556f32,
    ];

    let mut p = cartalith_engine::WorldParams::defaults(16, 12, 314159);
    p.world = true;
    p.climate.w_iters = 12;
    let ws = cartalith_engine::generate_terrain(&p);
    assert!((ws.sea_level - 0.42f64).abs() < 1e-9, "sea_level mismatch, harness assumption broken");

    let wb = cartalith_civ::build_water_bodies(&ws.field, 16, 12, ws.sea_level, true, Some(&ws.rainfall));

    assert_eq!(wb.classification, expected_classification, "classification");
    assert_close(&wb.fill_level, &expected_fill, "fill_level");
}
