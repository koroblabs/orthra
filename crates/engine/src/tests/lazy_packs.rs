//! A pack's dictionaries are read on the first word of its script, and not before: a language a
//! project enables and no document writes in costs nothing.

use std::sync::Arc;

use orthra_packs::Packs;

use crate::Engine;

#[test]
fn a_pack_is_read_on_the_first_word_of_its_script() {
    // Packs of the test's own: the shared ones are read by every other test.
    let packs = Arc::new(Packs::builtin(&["en-US"]).expect("en-US loads"));
    let mut engine = Engine::with_packs(Arc::clone(&packs));
    let english = packs.iter().next().expect("en-US");
    assert!(!english.is_loaded(), "enabling a pack reads nothing");

    engine.learn("rust", "fn recieve_message() {}\n");
    engine.check("plaintext", "Привіт, світе: нічого латинського.\n");
    assert!(
        !english.is_loaded(),
        "learning a project and checking a document with no Latin word read nothing"
    );

    engine.check("plaintext", "Teh end.\n");
    assert!(english.is_loaded(), "a Latin word reads the pack");
}
