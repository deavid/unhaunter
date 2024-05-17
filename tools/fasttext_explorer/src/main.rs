use fasttext::FastText;
use std::env;

fn main() {
    // Get the absolute path to the project root
    let project_root = env::current_dir().unwrap();
    let model_path = project_root.join("assets/phrasebooks/vectors/crawl-300d-2M-subword.bin");

    // Load the FastText model
    let mut model = FastText::new();
    model.load_model(model_path.to_str().unwrap()).unwrap();

    // Get the embedding for a word
    let word = "ghost";
    let embedding = model.get_word_vector(word).unwrap();
    println!("Embedding for '{}': {:?}", word, embedding);

    // Get the embedding for a sentence
    let sentence = "This is a haunted house.";
    let sentence_embedding = model.get_sentence_vector(sentence).unwrap();
    println!("Embedding for '{}': {:?}", sentence, sentence_embedding);
}
