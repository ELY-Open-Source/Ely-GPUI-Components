use super::*;

fn draft(to: &[&str], cc: &[&str], bcc: &[&str]) -> Draft {
    let list = |all: &[&str]| {
        all.iter()
            .map(|one| SharedString::from(one.to_string()))
            .collect()
    };
    Draft {
        to: list(to),
        cc: list(cc),
        bcc: list(bcc),
        subject: SharedString::default(),
    }
}

#[test]
fn a_message_goes_once_someone_holds_an_address() {
    let ben = "ben@atrium.studio";
    assert!(!ready(&draft(&[], &[], &[]), is_email), "nobody to send to");
    assert!(ready(&draft(&[ben], &[], &[]), is_email));
    assert!(
        ready(&draft(&[], &[], &[ben]), is_email),
        "a blind copy alone"
    );
    assert!(
        !ready(&draft(&[ben], &["ben@"], &[]), is_email),
        "an address that is not one"
    );
}

#[test]
fn typed_words_join_once() {
    let tags = [SharedString::from("a@b.c")];
    assert_eq!(with_typed(&tags, &" a@b.c, ".into()), tags);
    assert_eq!(with_typed(&tags, &"d@e.f,".into()), ["a@b.c", "d@e.f"]);
    assert_eq!(with_typed(&tags, &"  ".into()), tags);
}
