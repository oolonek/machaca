# machaca

Machaca checks whether the raw mass-spectrometry files named in a dataset's sample records are actually present. It helps catch missing or mismatched files before later analysis, without changing the dataset.

The name refers to the machaca, *Fulgora laternaria* ([Wikidata Q1934998](https://www.wikidata.org/wiki/Q1934998); [Catalogue of Life 6JQHB](https://www.catalogueoflife.org/data/taxon/6JQHB)), a lanternfly.

[![Fulgora laternaria specimen with its elongated head](https://commons.wikimedia.org/wiki/Special:FilePath/Fulgora%20laternaria%20TPopp.jpg?width=500)](https://commons.wikimedia.org/wiki/File:Fulgora_laternaria_TPopp.jpg)

Photo: [Tanja Popp](https://commons.wikimedia.org/wiki/File:Fulgora_laternaria_TPopp.jpg), [CC BY-SA 4.0](https://creativecommons.org/licenses/by-sa/4.0/).

## In the flock

Sapayoa downloads datasets; once an archive is extracted, Machaca compares its sample metadata with the raw files. This checks file references, not the identity of any chemical in a sample.

## Start here

For a small example of the full journey, follow [Mato's Zenodo toy-dataset walkthrough](../mato/TECHNICAL.md#example-download-the-zenodo-toy-dataset-and-export-a-kg). To run Machaca yourself, first extract the downloaded Zenodo ZIP outside this crate, then follow the [technical guide](TECHNICAL.md#check-an-extracted-dataset).

## Learn more

The [technical guide](TECHNICAL.md) explains interactive and scripted checks, which filenames count, what failures mean, MassIVE metadata checks, and the Rust interface.
