use std::io::BufRead;

use rualign::Aligner;

use vaporetto::Sentence;

fn main() {
    let mut sentences = vec![];
    for line in std::io::stdin().lock().lines() {
        let line = line.unwrap();
        sentences.push(Sentence::from_tokenized(&line).unwrap());
    }
    let mut aligner = Aligner::new(&sentences, 1);
    for i in 0..10 {
        let diff = aligner.update();
        eprintln!("iter {i}: log_diff = {diff}");
    }
    eprintln!("Finalizing...");
    let model = aligner.finalize();
    let mut buf = String::new();
    for mut sentence in sentences {
        model.make_alignment(&mut sentence, 1);
        sentence.write_tokenized_text(&mut buf);
        println!("{}", buf);
    }
}
