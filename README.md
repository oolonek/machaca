# machaca

Machaca checks whether the raw mass-spectrometry files named in a dataset's sample records are actually present. It helps catch missing or mismatched files before later analysis, without changing the dataset.

The flock turns mass-spectrometry samples and measurements into traceable evidence for possible chemical matches; it can also export a knowledge graph. Machaca checks that the dataset's named raw files are there before later work. Start with the [flock overview](https://github.com/oolonek/flock-roadmap/blob/main/README.md) to see how the tools fit together.

The name refers to the machaca, *Fulgora laternaria* ([Wikidata Q1934998](https://www.wikidata.org/wiki/Q1934998); [Catalogue of Life 6JQHB](https://www.catalogueoflife.org/data/taxon/6JQHB)), a lanternfly.

[![Fulgora laternaria specimen with its elongated head](https://commons.wikimedia.org/wiki/Special:FilePath/Fulgora%20laternaria%20TPopp.jpg?width=500)](https://commons.wikimedia.org/wiki/File:Fulgora_laternaria_TPopp.jpg)

Photo: [Tanja Popp](https://commons.wikimedia.org/wiki/File:Fulgora_laternaria_TPopp.jpg), [CC BY-SA 4.0](https://creativecommons.org/licenses/by-sa/4.0/).

## In the flock

Sapayoa downloads datasets; once an archive is extracted, Machaca compares its sample metadata with the raw files. This checks file references, not the identity of any chemical in a sample.

## Start here

For a small example after the overview, follow [Mato's Zenodo toy-dataset walkthrough](https://github.com/oolonek/mato/blob/master/TECHNICAL.md#example-download-the-zenodo-toy-dataset-and-export-a-kg). To run Machaca yourself, first extract the downloaded Zenodo ZIP outside this crate, then follow the [technical guide](TECHNICAL.md#check-an-extracted-dataset).

## Learn more

The [technical guide](TECHNICAL.md) explains interactive and scripted checks, which filenames count, what failures mean, MassIVE metadata checks, and the Rust interface.

Machaca is available under either the [MIT license](LICENSE-MIT) or the [Apache 2.0 license](LICENSE-APACHE).

*Naming note: For new species names, we take inspiration from [colonizer](https://crates.io/crates/colonizer)'s `inspire` mode, which suggests names from the Catalogue of Life, rather than inventing each crate name ourselves.*
