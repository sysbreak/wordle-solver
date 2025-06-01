# 🧠 Wordle Solver (Rust CLI)

[![Rust](https://img.shields.io/badge/Rust-Stable-orange?logo=rust)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
![Accuracy](https://img.shields.io/badge/Accuracy-100%25-brightgreen)

A command-line **Wordle solver** written in Rust. It narrows down the possible words using **entropy-based scoring** to suggest the most informative next guess based on feedback provided by the user.

---

## 📦 How to Use

### 1. **Build and Run**

Simply download the latest release and run it.

OR build it yourself

Make sure you have [Rust](https://www.rust-lang.org/tools/install) and `cargo` installed.

```bash
git clone https://github.com/sysbreak/wordle-solver.git
cd wordle-solver
cargo run 
```

### 2. **Play the Game**

- The program automatically fetches a Wordle word list from [draco's valid wordle word list](https://gist.github.com/dracos/dd0668f281e685bad51479e5acaadb93#file-valid-wordle-words-txt).
- Your first guess will be **`SOARE`** based on [Tom Johnston's study on Optimal Wordle strategies](https://tomjohnston.co.uk/blog/2022-02-07-optimal-wordle-strategies.html).
- After each guess, input the feedback pattern:

| Symbol | Meaning                |
|--------|------------------------|
| `@`    | Correct letter, correct position (green) |
| `#`    | Correct letter, wrong position (yellow) |
| `?`    | Incorrect letter (gray) |

Example: If your guess was `SOARE` and the feedback from Wordle was:
- S and E are green
- R is yellow
- O and A are gray

You would input: `@??#@`

- If, for some reason, the word given to you by the solver comes up as a non-existant word, type `reject` to get a new guess from the solver.
- After 6 guesses or finding the correct word (`@@@@@`), the game ends.

---

## ⚙️ How It Works

### 🧮 Entropy-Based Guessing

At each iteration, the solver exhaustively evaluates the entire candidate word set in a two-tiered nested loop, yielding an overall computational complexity of O(n²), where n is the number of remaining viable words.
1. For each candidate guess word, the algorithm simulates hypothetical feedback scenarios by comparing it against every other candidate word in the current solution space.
2. This exhaustive pairwise comparison constructs a probabilistic distribution of feedback patterns, which is then used to compute the expected information gain via entropy metrics.
3. The guess that maximizes this expected reduction in uncertainty—i.e., the highest Shannon entropy gain—is selected as the next move.

This approach strategically maximizes information-theoretic efficiency, enabling the solver to minimize the expected number of guesses by focusing on hypotheses that partition the solution space most effectively.

---

## 📝 License

MIT License 

---

## 🤝 Contributions

Issues and PRs welcome! Add new features like:
- GUI support
- Automated playing
- Solver benchmarking

