# Another layout generator

This is hopefully last rewrite of my generator. This time I cut everything I
could out of it. I could have made it shorter, but I've decided to also try
usign 0 dependencies, which came out surprisingly well (did not result in
a disaster).

Like my first version (which was unusable by anyone but me) this generator
focuses on exhaustive search of keyboard layouts with low sfbs. I wanted this
generator to be usable with custom metrics and didn't want to embed a custom
language into it, so I wrote a "json parser" to read bigram weights from a file.

## Example usage

With cargo:
cargo run -- --precalculated --file monkeyracer_sfbs.json --cutoff 0.006

Without `--file` the weights / corpus is loaded through stdio. Without
`--precalculated` generator calcualtes weights based on provided text as `4 *
sfbs + dsfbs`. `--alphabet` sets (if none is present) or overrides characters
placed on a keyboard.

## Theory

To make exhaustive search feasible we are using two optimizations.

First, we are exploiting various symmetries. We don't distinguish between layouts
with columns swapped or with keys swapped inside a column.

Second, instead of trying to go through all possible layouts ony by one, we look
at groups of layouts instead. Specifically, we place characters on a keyboard
in order of frequency untill we get complete layout or we cross the cutoff, at
which point the whole group gets pruned. Because it's impossible for a layout
to have less sfbs after adding another letter to it, we can be sure that all
layouts we skip are above the cutoff point.


## Fun stats

Number of column sets below percentage of sfbs on monkeyracer corpus. Weights
taken from [cmini repo](https://github.com/Apsu/cmini/). Percentages are
normalized to bigrams containing only letters found on the layout.

|  %  | column sets |
|-----|-------------|
| 0.5 |          70 |
| 0.6 |       14638 |
| 0.7 |      410645 |
| 0.8 |     5584638 |
| 0.9 |    48430687 |
| 1.0 |   299938617 |
