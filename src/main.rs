use itertools::Itertools;
use std::env;
use std::io::prelude::*;
use std::process::{Command, Stdio};

const TMP_MIDI_FILE_PATH: &str = "tmp";

// ly_code="\\language $language \\score { \\relative $rel_note { $notes } \\midi {} }"

fn main() -> Result<(), &'static str> {
    // Get arguments
    let (note_language, relative_note, notes_to_play) = match env::args().skip(1).next_tuple() {
        Some(tpl) => tpl,
        None => {
            return Err(concat!(
                "Exactly three arguments needed!\n",
                "1: notes language",
                "2: relative note",
                "3: notes to play",
            ))
        }
    };

    // Generate lilypond code wrapping notes to play
    let ly_code_to_exe = format!(
        r#"
\language "{}"
\score {{
    \relative {} {{
        {}
    }}
    \midi {{}}
}}"#,
        note_language, relative_note, notes_to_play,
    );

    // The generated lilypond code is used to gnerate the midi file that will be played
    let lyproc = Command::new("lilypond")
        // TODO: remove need to create tmp midi file
        .args(&["-o", TMP_MIDI_FILE_PATH]) 
        // So that the output is read from `stdin`
        .arg("-")
        .stdin(Stdio::piped())
        // No terminal polution
        .stderr(Stdio::null())
        .stdout(Stdio::null())
        // Spawned process waits for input on `stdin`
        .spawn()
        .expect("Error while running lilypond command to generate midi");

    // Produced lilycode code is piped to running lilypond process
    lyproc
        .stdin
        .unwrap()
        // Lilypad code is passed on lyproc stdin
        .write_all(ly_code_to_exe.as_bytes())
        .unwrap();

    // Wait for file to be created
    // TODO: ugly hack... change this in the future
    while let Err(_) = std::fs::read_to_string("tmp.midi") {}

    // Use wildmidi to play the produced sample
    Command::new("wildmidi")
        .stdout(Stdio::null())
        .arg(format!("{}.midi", TMP_MIDI_FILE_PATH))
        .output()
        .expect("Could not run wildmidi");

    // All is fine
    Ok(())
}
