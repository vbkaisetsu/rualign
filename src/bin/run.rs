use std::io::BufRead;

use rualign::Aligner;

use vaporetto::Sentence;

fn main() {
    let mut sentences = vec![];
    for line in std::io::stdin().lock().lines() {
        let line = line.unwrap();
        sentences.push(Sentence::from_tokenized(&line).unwrap());
    }
    eprintln!("Initializing...");
    let mut aligner = Aligner::new(&sentences, 1);
    eprintln!("Training...");
    for i in 0..20 {
        let log_diff = aligner.update();
        eprintln!("#{i} log_diff: {log_diff}");
        if log_diff < -20.0 {
            break;
        }
    }
    eprintln!("Finalizing...");
    let phoneme_map = aligner.finalize();
    let mut buf = String::new();
    for mut sentence in sentences {
        phoneme_map.make_alignment(&mut sentence, 1);
        sentence.write_tokenized_text(&mut buf);
        println!("{}", buf);
    }
}
