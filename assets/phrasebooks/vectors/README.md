# FastText Word Embeddings for Unhaunter

This folder contains pre-trained word embeddings generated using the FastText 
library. These embeddings are used to power the ghost communication system in
Unhaunter, allowing for a more dynamic and emergent gameplay experience.

## Model Source

The word embeddings in this folder were downloaded from the FastText website:

* **Model:** crawl-300d-2M-subword.zip
* **Description:** 2 million word vectors trained with subword information on Common Crawl (600B tokens).
* **Download Link:** https://fasttext.cc/docs/en/english-vectors.html

Unzip the archive, find and place `crawl-300d-2M-subword.bin` in this folder.

## Attribution

The FastText library and the pre-trained models are developed and provided by Facebook AI Research.

Please cite the following paper if you use these embeddings in your own work:

```
T. Mikolov, E. Grave, P. Bojanowski, C. Puhrsch, A. Joulin. Advances in Pre-Training Distributed Word Representations

@inproceedings{mikolov2018advances,
  title={Advances in Pre-Training Distributed Word Representations},
  author={Mikolov, Tomas and Grave, Edouard and Bojanowski, Piotr and Puhrsch, Christian and Joulin, Armand},
  booktitle={Proceedings of the International Conference on Language Resources and Evaluation (LREC 2018)},
  year={2018}
}
```

## License

Note that we do not ship the word embeddings, and these have a different license
from the game. At the time of download it was 
"Creative Commons Attribution-Share-Alike License 3.0".

Please check when downloading the actual license used in case it changes later.

## Snapshot and Hashes

To help on the integrity of the file, the one the author downloaded had the
following properties: (After unzipping, just the bin file)

`7235845152 bytes, Aug 22  2018, crawl-300d-2M-subword.bin`

sha1sum: `e6b07293f7b0095e3c72c2a12bc09464b69444b0`
sha256sum: `ab4bb2a7660bcf441366055fc49a10ce5dcefbf0fcfc0e588e33c08d0fe077da`

NOTE: If you are unable to find this file and version, don't despair. If you can
    find any word vectors for English for FastText, you could use that instead. 
    However you'll need to recompute all the embeddings in this case to ensure
    consistency.

## Usage in Unhaunter

These word embeddings are **not required to play the game.**  
They are used during development to analyze the semantic similarity of
phrases and to generate ghost responses.  

If you are contributing to the development of Unhaunter and need to modify 
the phrasebooks, you will need to download the FastText model and generate new
embeddings for any added phrases.

TODO: Add documentation on how to regenerate the embeddings.