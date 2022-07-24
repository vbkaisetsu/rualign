//! Character-wise automatic pronunciation alignment algorithm.
//!
//! Reference:
//! Keigo Kubo, Hiromichi Kawanami, Hiroshi Saruwatari and Kiyohiro Shikano.
//! Evaluation of Many-to-Many Alignment Algorithm by Automatic Pronunciation
//! Annotation Using Web Text Mining. INTERSPEECH 2012.

#![no_std]

#[macro_use]
extern crate alloc;

mod array_2d;
mod map;

use alloc::{string::String, vec::Vec};

use hashbrown::{HashMap, HashSet};
use vaporetto::{CharacterBoundary, Sentence};

use array_2d::Array2d;
use map::{HashMap2, HashSet4};

fn logsumexp(a: f64, b: f64) -> f64 {
    if a > b {
        a + ((b - a).exp() + 1.0).ln()
    } else {
        b + ((a - b).exp() + 1.0).ln()
    }
}

fn log_square_error(a: f64, b: f64) -> f64 {
    if a > b {
        (a + (1.0 - (b - a).exp()).ln()) * 2.0
    } else {
        (b + (1.0 - (a - b).exp()).ln()) * 2.0
    }
}

pub struct Aligner {
    dataset: Vec<(Vec<char>, Vec<char>)>,
    alphas: Array2d<f64>,
    betas: Array2d<f64>,
    scores: HashMap2<Vec<char>, Vec<char>, f64>,
}

impl Aligner {
    pub fn new(sentences: &[Sentence], tag_index: usize) -> Self {
        let mut dataset: Vec<(Vec<char>, Vec<char>)> = vec![];
        for sentence in sentences {
            for token in sentence.iter_tokens() {
                let phoneme = token
                    .tags()
                    .get(tag_index)
                    .and_then(|x| x.as_ref())
                    .map_or("", |x| x.as_ref());
                dataset.push((token.surface().chars().collect(), phoneme.chars().collect()));
            }
        }

        // Initializes scores
        let mut scores = HashMap2::new();
        let mut cnt = 0;
        for (surface, phoneme) in &dataset {
            cnt += surface.len() * phoneme.len();
        }
        let init_score = -(cnt as f64).ln();
        for (surface, phoneme) in &dataset {
            for i in 0..surface.len() {
                for j in 0..phoneme.len() + 1 {
                    if i == 0 && j != 0 {
                        continue;
                    }
                    for p in i + 1..surface.len() + 1 {
                        for q in j..phoneme.len() + 1 {
                            if p == surface.len() && q != phoneme.len() {
                                continue;
                            }
                            scores.insert(
                                surface[i..p].to_vec(),
                                phoneme[j..q].to_vec(),
                                init_score,
                            );
                        }
                    }
                }
            }
        }

        Self {
            dataset,
            alphas: Array2d::new(0, 0),
            betas: Array2d::new(0, 0),
            scores,
        }
    }

    fn calculate_alphas(
        surface: &[char],
        phoneme: &[char],
        scores: &HashMap2<Vec<char>, Vec<char>, f64>,
        alphas: &mut Array2d<f64>,
    ) {
        alphas.resize(surface.len() + 1, phoneme.len() + 1, f64::NEG_INFINITY);
        alphas.fill(f64::NEG_INFINITY);
        *alphas.get_mut(0, 0).unwrap() = 0.0;
        for i in 0..surface.len() {
            for j in 0..phoneme.len() + 1 {
                if i == 0 && j != 0 {
                    continue;
                }
                for p in i + 1..surface.len() + 1 {
                    for q in j..phoneme.len() + 1 {
                        if p == surface.len() && q != phoneme.len() {
                            continue;
                        }
                        let score = *scores.get(&surface[i..p], &phoneme[j..q]).unwrap();
                        let distance = (p - i + (q - j).max(1)) as f64;
                        *alphas.get_mut(p, q).unwrap() = logsumexp(
                            *alphas.get(p, q).unwrap(),
                            *alphas.get(i, j).unwrap() + score * distance,
                        );
                    }
                }
            }
        }
    }

    fn calculate_betas(
        surface: &[char],
        phoneme: &[char],
        scores: &HashMap2<Vec<char>, Vec<char>, f64>,
        betas: &mut Array2d<f64>,
    ) {
        betas.resize(surface.len() + 1, phoneme.len() + 1, f64::NEG_INFINITY);
        betas.fill(f64::NEG_INFINITY);
        *betas.get_mut(surface.len(), phoneme.len()).unwrap() = 0.0;
        for i in (0..surface.len()).rev() {
            for j in (0..phoneme.len() + 1).rev() {
                if i == 0 && j != 0 {
                    continue;
                }
                for p in (i + 1..surface.len() + 1).rev() {
                    for q in (j..phoneme.len() + 1).rev() {
                        if p == surface.len() && q != phoneme.len() {
                            continue;
                        }
                        let score = *scores.get(&surface[i..p], &phoneme[j..q]).unwrap();
                        let distance = (p - i + (q - j).max(1)) as f64;
                        *betas.get_mut(i, j).unwrap() = logsumexp(
                            *betas.get(i, j).unwrap(),
                            *betas.get(p, q).unwrap() + score * distance,
                        );
                    }
                }
            }
        }
    }

    fn calculate_gammas<'a, 'b>(
        surface: &'a [char],
        phoneme: &'b [char],
        scores: &HashMap2<Vec<char>, Vec<char>, f64>,
        alphas: &Array2d<f64>,
        betas: &Array2d<f64>,
        gammas: &mut HashMap2<&'a [char], &'b [char], f64>,
    ) {
        let score_sum = *betas.get(0, 0).unwrap();
        for i in (0..surface.len()).rev() {
            for j in (0..phoneme.len() + 1).rev() {
                if i == 0 && j != 0 {
                    continue;
                }
                for p in (i + 1..surface.len() + 1).rev() {
                    for q in (j..phoneme.len() + 1).rev() {
                        if p == surface.len() && q != phoneme.len() {
                            continue;
                        }
                        let surface_slice = &surface[i..p];
                        let phoneme_slice = &phoneme[j..q];
                        let score = *scores.get(surface_slice, phoneme_slice).unwrap();
                        let distance = (p - i + (q - j).max(1)) as f64;
                        let gamma = logsumexp(
                            *gammas
                                .get(surface_slice, phoneme_slice)
                                .unwrap_or(&f64::NEG_INFINITY),
                            *alphas.get(i, j).unwrap()
                                + *betas.get(p, q).unwrap()
                                + score * distance
                                - score_sum,
                        );
                        gammas.insert(surface_slice, phoneme_slice, gamma);
                    }
                }
            }
        }
    }

    fn search_best_path<'a>(
        scores: &HashMap2<Vec<char>, Vec<char>, f64>,
        surface: &'a [char],
        phoneme: &'a [char],
        best_nodes: &mut Array2d<(f64, usize, usize)>,
    ) -> Vec<(usize, usize)> {
        best_nodes.fill((f64::NEG_INFINITY, 0, 0));
        best_nodes.resize(
            surface.len() + 1,
            phoneme.len() + 1,
            (f64::NEG_INFINITY, 0, 0),
        );
        best_nodes.get_mut(surface.len(), phoneme.len()).unwrap().0 = 0.0;
        for i in (0..surface.len()).rev() {
            for j in (0..phoneme.len() + 1).rev() {
                if i == 0 && j != 0 {
                    continue;
                }
                for p in (i + 1..surface.len() + 1).rev() {
                    for q in (j..phoneme.len() + 1).rev() {
                        if p == surface.len() && q != phoneme.len() {
                            continue;
                        }
                        let score = *scores.get(&surface[i..p], &phoneme[j..q]).unwrap();
                        let distance = (p - i + (q - j).max(1)) as f64;
                        let new_score = best_nodes.get(p, q).unwrap().0 + score * distance;
                        let current_best_node = best_nodes.get_mut(i, j).unwrap();
                        if current_best_node.0 < new_score {
                            *current_best_node = (new_score, p, q);
                        }
                    }
                }
            }
        }
        let mut result = vec![];
        let (mut i, mut j) = (0, 0);
        while i != surface.len() && j != phoneme.len() {
            let (_, next_i, next_j) = *best_nodes.get(i, j).unwrap();
            result.push((next_i, next_j));
            i = next_i;
            j = next_j;
        }
        result
    }

    fn merge_phonemes(phoneme_map: &mut HashMap2<Vec<char>, Vec<char>, Vec<(usize, usize)>>) {
        let mut alignment_next = HashMap::new();
        let mut alignment_prev = HashMap::new();
        phoneme_map.for_each(|(surface, phoneme, alignments)| {
            let mut surface_start_pos = 0;
            let mut phoneme_start_pos = 0;
            let mut surf_phoneme = vec![];
            for &(surface_end_pos, phoneme_end_pos) in alignments {
                surf_phoneme.push((
                    &surface[surface_start_pos..surface_end_pos],
                    &phoneme[phoneme_start_pos..phoneme_end_pos],
                ));
                surface_start_pos = surface_end_pos;
                phoneme_start_pos = phoneme_end_pos;
            }
            for i in 0..surf_phoneme.len() {
                if i != 0 {
                    alignment_prev
                        .entry(surf_phoneme[i])
                        .or_insert_with(HashSet::new)
                        .insert(surf_phoneme[i - 1]);
                }
                if i != surf_phoneme.len() - 1 {
                    alignment_next
                        .entry(surf_phoneme[i])
                        .or_insert_with(HashSet::new)
                        .insert(surf_phoneme[i + 1]);
                }
            }
        });
        let mut mergeable_alignment = HashSet4::new();
        for ((surface, phoneme), next) in alignment_next {
            if next.len() == 1 {
                let (surface_next, phoneme_next) = next.into_iter().next().unwrap();
                mergeable_alignment.insert(
                    surface.to_vec(),
                    phoneme.to_vec(),
                    surface_next.to_vec(),
                    phoneme_next.to_vec(),
                );
            }
        }
        for ((surface, phoneme), prev) in alignment_prev {
            if prev.len() == 1 {
                let (surface_prev, phoneme_prev) = prev.into_iter().next().unwrap();
                mergeable_alignment.insert(
                    surface_prev.to_vec(),
                    phoneme_prev.to_vec(),
                    surface.to_vec(),
                    phoneme.to_vec(),
                );
            }
        }

        phoneme_map.for_each_mut(|(surface, phoneme, alignments_new)| {
            let alignments = core::mem::take(alignments_new);
            let mut surface_start_pos = 0;
            let mut phoneme_start_pos = 0;
            for (surface_end_pos, phoneme_end_pos) in alignments {
                if let Some((surface_middle_pos, phoneme_middle_pos)) = alignments_new.last_mut() {
                    if mergeable_alignment.contains(
                        &surface[surface_start_pos..*surface_middle_pos],
                        &phoneme[phoneme_start_pos..*phoneme_middle_pos],
                        &surface[*surface_middle_pos..surface_end_pos],
                        &phoneme[*phoneme_middle_pos..phoneme_end_pos],
                    ) {
                        *surface_middle_pos = surface_end_pos;
                        *phoneme_middle_pos = phoneme_end_pos;
                        continue;
                    }
                    surface_start_pos = *surface_middle_pos;
                    phoneme_start_pos = *phoneme_middle_pos;
                }
                alignments_new.push((surface_end_pos, phoneme_end_pos));
            }
        });
    }

    pub fn update(&mut self) -> f64 {
        // Scores calculated in E-step
        let mut gammas = HashMap2::new();

        // E-step
        for (surface, phoneme) in &self.dataset {
            // The original algorithm divides training into the first and second parts to
            // prevent the excessive occurance of deletion characters from being generated
            // caused by the city block distance. The first part uses the EM algorithm to train
            // alignments excluding the deletion character, and the second part uses the n-best
            // Viterbi training to learn the deletion character.
            //
            // In contrast, this implementation adds the cost corresponding to the deletion
            // characters to the city block distance from the beginning to simplify the
            // algorithm while preventing the excessive occurrence of deletion characters.
            Self::calculate_alphas(surface, phoneme, &self.scores, &mut self.alphas);
            Self::calculate_betas(surface, phoneme, &self.scores, &mut self.betas);
            Self::calculate_gammas(
                surface,
                phoneme,
                &self.scores,
                &self.alphas,
                &self.betas,
                &mut gammas,
            );
        }

        // M-step
        let mut diff_total = f64::NEG_INFINITY;
        let mut gamma_sum = f64::NEG_INFINITY;
        gammas.for_each(|(_, _, &v)| {
            gamma_sum = logsumexp(gamma_sum, v);
        });
        gammas.for_each(|(&k1, &k2, &v)| {
            let score = self.scores.get_mut(k1, k2).unwrap();
            diff_total = logsumexp(diff_total, log_square_error(v - gamma_sum, *score));
            *score = v - gamma_sum;
        });

        diff_total
    }

    pub fn scores(&self) -> &HashMap2<Vec<char>, Vec<char>, f64> {
        &self.scores
    }

    pub fn finalize(self) -> Model {
        // Searches the best paths
        let mut best_nodes = Array2d::new(0, 0);
        let mut phoneme_map = HashMap2::new();
        for (surface, phoneme) in self.dataset {
            if phoneme_map.contains_key(&surface, &phoneme) {
                continue;
            }
            let result = Self::search_best_path(&self.scores, &surface, &phoneme, &mut best_nodes);
            phoneme_map.insert(surface, phoneme, result);
        }

        Self::merge_phonemes(&mut phoneme_map);

        Model { phoneme_map }
    }
}

pub struct Model {
    phoneme_map: HashMap2<Vec<char>, Vec<char>, Vec<(usize, usize)>>,
}

impl Model {
    pub fn make_alignment(&self, sentence: &mut Sentence, tag_index: usize) {
        let mut new_boundaries = vec![];
        for token in sentence.iter_tokens() {
            let phoneme = token
                .tags()
                .get(tag_index)
                .and_then(|x| x.as_ref())
                .map_or("", |x| x.as_ref());
            let surface: Vec<_> = token.surface().chars().collect();
            let phoneme: Vec<_> = phoneme.chars().collect();
            let mut phoneme_start_pos = 0;
            for &(surface_end_pos, phoneme_end_pos) in
                self.phoneme_map.get(&surface, &phoneme).unwrap()
            {
                let phoneme: String = phoneme[phoneme_start_pos..phoneme_end_pos].iter().collect();
                new_boundaries.push((token.start() + surface_end_pos - 1, phoneme));
                phoneme_start_pos = phoneme_end_pos;
            }
        }
        sentence.reset_tags(1);
        for (pos, tag) in new_boundaries {
            if pos != sentence.boundaries().len() {
                sentence.boundaries_mut()[pos] = CharacterBoundary::WordBoundary;
            }
            sentence.tags_mut()[pos].replace(tag.into());
        }
    }
}
