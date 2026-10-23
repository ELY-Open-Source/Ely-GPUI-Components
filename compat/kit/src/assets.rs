use ely_gpui_component::Assets;
use gpui_kit::AssetSource;

#[test]
fn one_source_serves_both_libraries() {
    let kit = gpui_kit::assets::Assets;
    let both = Assets::before(gpui_kit::assets::Assets);
    let ely = Assets.list("").unwrap();
    assert!(
        ely.iter().any(|path| path.ends_with(".ttf")),
        "Ely bundles its fonts: {ely:?}"
    );
    for path in &ely {
        let served = both.load(path).unwrap().expect("Ely's file loads");
        assert_eq!(
            served,
            Assets.load(path).unwrap().unwrap(),
            "{path} is Ely's"
        );
    }
    let icons = kit.list("icons/").unwrap();
    assert!(!icons.is_empty(), "Kit lists its icons");
    for path in &icons {
        let served = both.load(path).unwrap().expect("Kit's icon loads");
        assert_eq!(served, kit.load(path).unwrap().unwrap(), "{path} is Kit's");
    }
}
