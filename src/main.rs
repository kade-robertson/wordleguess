use std::{
    collections::{HashMap, HashSet},
    io::{self, Write},
};

use text_io::read;

fn get_letter_frequency(wordlist: &Vec<String>) -> HashMap<char, u16> {
    let mut charmap = HashMap::new();

    wordlist.iter().for_each(|word| {
        word.chars().for_each(|c| {
            if charmap.contains_key(&c) {
                (*charmap.get_mut(&c).unwrap()) += 1;
            } else {
                charmap.insert(c, 1);
            }
        })
    });

    charmap
}

fn get_word_score(word: &String, letter_frequency: &HashMap<char, u16>) -> u16 {
    let mut seen_chars: HashSet<char> = HashSet::new();
    let mut score = 0;

    word.chars().for_each(|c| match seen_chars.contains(&c) {
        false => {
            score += letter_frequency[&c];
            seen_chars.insert(c);
        }
        true => (),
    });

    score
}

fn main() {
    let mut wordlist = include_str!("words.txt")
        .to_string()
        .lines()
        .map(|line| line.to_lowercase())
        .filter(|word| word.len() == 5 && word.chars().all(char::is_alphabetic))
        .collect::<Vec<String>>();

    let letter_frequency = get_letter_frequency(&wordlist);

    wordlist.sort_by(|w1, w2| {
        get_word_score(w2, &letter_frequency).cmp(&get_word_score(w1, &letter_frequency))
    });

    loop {
        let current_word = &wordlist[0];

        println!(
            "Current guess is: {} ({} words left)",
            current_word,
            wordlist.len()
        );
        println!("What was the result?");
        println!("- Enter a 0 if the letter does not appear in the word.");
        println!("- Enter a 1 if the letter does appear in the word, but not in that spot.");
        println!("- Enter a 2 if the letter was correct.");
        print!("> ");
        io::stdout().flush().unwrap();

        let response: String = read!("{}\n");
        let response_score = response
            .chars()
            .map(|c| match c {
                '1' => 1,
                '2' => 2,
                _ => 0,
            })
            .collect::<Vec<u8>>();

        wordlist = wordlist
            .iter()
            .filter(|word| {
                current_word
                    .chars()
                    .zip(response_score.clone())
                    .enumerate()
                    .all(|(i, (c, score))| match score {
                        0 => !word.contains(c),
                        1 => word.contains(c) && word.find(c) != Some(i),
                        _ => word.find(c) == Some(i),
                    })
            })
            .map(|word| word.to_owned())
            .collect();

        if wordlist.len() < 25 {
            println!("{:?}", wordlist);
        }

        if wordlist.len() == 0 {
            break;
        }
    }
}
