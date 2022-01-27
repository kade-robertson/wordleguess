use std::{
    collections::{HashMap, HashSet},
    io::{self, Write},
};

use colored::Colorize;
use float_ord::{self, FloatOrd};
use text_io::read;

pub trait EasyUpdate<K, V> {
    fn update(&mut self, key: K, value: V);
}

impl<K: Eq + core::hash::Hash, V: std::ops::AddAssign> EasyUpdate<K, V> for HashMap<K, V> {
    fn update(&mut self, k: K, v: V) {
        if self.contains_key(&k) {
            (*self.get_mut(&k).unwrap()) += v;
        } else {
            self.insert(k, v);
        }
    }
}

/**
 * words-scored.txt contains the entire wordlist of words.txt, along with a score.
 *
 * The score was calculated by finding the mean popularity of a word from 1980 to 2022.
 *
 * Then, it was multiplied by 10 ^ 11 to get all values above 1, and then it was additionally
 * scaled by figuring out the lowest value (~0.88), figuring out what it would take to scale this
 * to be at least 10 (~11.314), and multiplying all values by that amount. This was to make
 * taking log_10 later easier, as we'd be guaranteed all word popularity factors were >= 1.
 */
fn get_wordlist() -> Vec<(String, FloatOrd<f64>)> {
    include_str!("words-scored.txt")
        .to_string()
        .lines()
        .map(|line| {
            let line_split = line.split(',').collect::<Vec<_>>();
            (
                line_split[0].to_string(),
                FloatOrd(line_split[1].parse::<f64>().unwrap_or(10f64).log10()),
            )
        })
        .collect()
}

fn get_letter_frequency(wordlist: &Vec<(String, FloatOrd<f64>)>) -> HashMap<char, u16> {
    let mut charmap = HashMap::new();

    wordlist.iter().for_each(|(word, _)| {
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

fn get_word_score(word: &String, letter_frequency: &HashMap<char, u16>) -> f64 {
    let mut seen_chars: HashSet<char> = HashSet::new();
    let mut score = 0;

    word.chars().for_each(|c| match seen_chars.contains(&c) {
        false => {
            score += letter_frequency[&c];
            seen_chars.insert(c);
        }
        true => (),
    });

    score as f64
}

fn main() {
    let wordlist = get_wordlist();

    let mut cloned = wordlist.clone();
    cloned.sort_by(|w1, w2| w2.1.cmp(&w1.1));

    let letter_frequency = get_letter_frequency(&wordlist);

    let mut scored_wordlist = wordlist
        .iter()
        .map(|(word, ngram_score)| {
            (
                word.to_owned(),
                FloatOrd(get_word_score(word, &letter_frequency) * ngram_score.0),
            )
        })
        .collect::<Vec<(String, FloatOrd<f64>)>>();

    scored_wordlist.sort_by(|w1, w2| w2.1.cmp(&w1.1));

    loop {
        let current_word = &scored_wordlist[0].0;

        println!(
            "{} {} ({} words left)",
            "Current guess is:".cyan().bold(),
            current_word,
            scored_wordlist.len()
        );
        println!("What was the result?");
        println!(
            "- Enter a {} if the letter does not appear in the word.",
            "0".red()
        );
        println!(
            "- Enter a {} if the letter does appear in the word, but not in that spot.",
            "1".yellow().bold()
        );
        println!(
            "- Enter a {} if the letter was correct.",
            "2".green().bold()
        );
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

        let mut letter_occurrences: HashMap<char, i8> = HashMap::new();

        current_word.chars().enumerate().for_each(|(i, c)| {
            letter_occurrences.update(c, if response_score[i] > 0 { 1 } else { 0 })
        });

        let mut chars_by_score = current_word
            .chars()
            .zip(response_score.clone())
            .enumerate()
            .collect::<Vec<(usize, (char, u8))>>();

        chars_by_score.sort_by(|z1, z2| z2.1 .1.cmp(&z1.1 .1));

        scored_wordlist = scored_wordlist
            .iter()
            .filter(|word| {
                let mut allowed_occurrences = letter_occurrences.clone();

                chars_by_score.iter().all(|(i, (c, score))| {
                    let char_indices = word.0.match_indices(*c);
                    let char_at_pos = |idx| char_indices.clone().find(|m| m.0 == idx).is_some();
                    let char_count = char_indices.clone().count();
                    let result = match score {
                        0 => char_count == 0 || allowed_occurrences[c] > 0,
                        1 => {
                            if char_count > 0 && !char_at_pos(*i) {
                                allowed_occurrences.update(*c, -1);
                                true
                            } else {
                                false
                            }
                        }
                        _ => {
                            if char_at_pos(*i) {
                                allowed_occurrences.update(*c, -1);
                                true
                            } else {
                                false
                            }
                        }
                    };
                    result
                })
            })
            .map(|word| word.to_owned())
            .collect();

        println!("");
        match scored_wordlist.len() {
            2..=10 => {
                println!(
                    "{} {}",
                    "Remaining options:".cyan().bold(),
                    scored_wordlist
                        .iter()
                        .map(|(w, _s)| w.to_owned())
                        .collect::<Vec<_>>()
                        .join(", ")
                        .magenta()
                        .bold()
                );
            }
            1 => {
                println!("Correct word is: {}", scored_wordlist[0].0.green().bold());
                break;
            }
            0 => {
                println!("Couldn't find a match :(");
                break;
            }
            _ => (),
        }
    }
}
