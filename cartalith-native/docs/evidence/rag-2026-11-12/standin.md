#### world 71077345: stand-in:trigram-hash-128

Model `stand-in:trigram-hash-128` (dimension 128), 1117 units, 64 questions. Verdict: **NOT NEEDED - lift on the paraphrase/miss subset is 4.2 points, below the 10 required; exact-name recall@10 falls 50.0 points, above the 1 allowed**.

Paraphrase / miss subset (16 questions): lexical recall@10 0.198, hybrid 0.240, lift +4.2 points (bar: at least 10). Exact-name loss +50.0 points (bar: at most 1). Overall recall@10 lexical 0.767, hybrid 0.452. Questions without a query vector: 0.

| Category | With gold | Lexical misses | Lexical R@10 | Hybrid R@10 |
|---|---:|---:|---:|---:|
| conflict | 5 | 0 | 1.000 | 0.500 |
| exact_name | 11 | 0 | 1.000 | 0.500 |
| figure | 10 | 0 | 1.000 | 0.000 |
| multi_hop | 8 | 8 | 0.396 | 0.354 |
| paraphrase | 8 | 8 | 0.000 | 0.125 |
| relational | 3 | 0 | 1.000 | 1.000 |
| story | 10 | 0 | 1.000 | 1.000 |

Index size: 571904 bytes as f32 (0.55 MiB). Embedding throughput - and VRAM spill - (not measured: no live embedding run).

#### world 71077345: control:random-128

Model `control:random-128` (dimension 128), 1117 units, 64 questions. Verdict: **NOT NEEDED - lift on the paraphrase/miss subset is -4.2 points, below the 10 required**.

Paraphrase / miss subset (16 questions): lexical recall@10 0.198, hybrid 0.156, lift -4.2 points (bar: at least 10). Exact-name loss +0.0 points (bar: at most 1). Overall recall@10 lexical 0.767, hybrid 0.755. Questions without a query vector: 0.

| Category | With gold | Lexical misses | Lexical R@10 | Hybrid R@10 |
|---|---:|---:|---:|---:|
| conflict | 5 | 0 | 1.000 | 1.000 |
| exact_name | 11 | 0 | 1.000 | 1.000 |
| figure | 10 | 0 | 1.000 | 1.000 |
| multi_hop | 8 | 8 | 0.396 | 0.312 |
| paraphrase | 8 | 8 | 0.000 | 0.000 |
| relational | 3 | 0 | 1.000 | 1.000 |
| story | 10 | 0 | 1.000 | 1.000 |

Index size: 571904 bytes as f32 (0.55 MiB). Embedding throughput - and VRAM spill - (not measured: no live embedding run).

#### world 71077345: control:gold-biased-128

Model `control:gold-biased-128` (dimension 128), 1117 units, 64 questions. Verdict: **ADOPT (both conditions hold)**.

Paraphrase / miss subset (16 questions): lexical recall@10 0.198, hybrid 0.896, lift +69.8 points (bar: at least 10). Exact-name loss +0.0 points (bar: at most 1). Overall recall@10 lexical 0.767, hybrid 0.970. Questions without a query vector: 0.

| Category | With gold | Lexical misses | Lexical R@10 | Hybrid R@10 |
|---|---:|---:|---:|---:|
| conflict | 5 | 0 | 1.000 | 1.000 |
| exact_name | 11 | 0 | 1.000 | 1.000 |
| figure | 10 | 0 | 1.000 | 1.000 |
| multi_hop | 8 | 8 | 0.396 | 0.792 |
| paraphrase | 8 | 8 | 0.000 | 1.000 |
| relational | 3 | 0 | 1.000 | 1.000 |
| story | 10 | 0 | 1.000 | 1.000 |

Index size: 571904 bytes as f32 (0.55 MiB). Embedding throughput - and VRAM spill - (not measured: no live embedding run).

#### world 24601: stand-in:trigram-hash-128

Model `stand-in:trigram-hash-128` (dimension 128), 1033 units, 66 questions. Verdict: **NOT NEEDED - lift on the paraphrase/miss subset is 4.2 points, below the 10 required; exact-name recall@10 falls 45.5 points, above the 1 allowed**.

Paraphrase / miss subset (16 questions): lexical recall@10 0.198, hybrid 0.240, lift +4.2 points (bar: at least 10). Exact-name loss +45.5 points (bar: at most 1). Overall recall@10 lexical 0.775, hybrid 0.497. Questions without a query vector: 0.

| Category | With gold | Lexical misses | Lexical R@10 | Hybrid R@10 |
|---|---:|---:|---:|---:|
| conflict | 5 | 0 | 1.000 | 0.500 |
| exact_name | 11 | 0 | 1.000 | 0.545 |
| figure | 10 | 0 | 1.000 | 0.100 |
| multi_hop | 8 | 8 | 0.396 | 0.354 |
| paraphrase | 8 | 8 | 0.000 | 0.125 |
| relational | 5 | 0 | 1.000 | 1.000 |
| story | 10 | 0 | 1.000 | 1.000 |

Index size: 528896 bytes as f32 (0.50 MiB). Embedding throughput - and VRAM spill - (not measured: no live embedding run).

#### world 24601: control:random-128

Model `control:random-128` (dimension 128), 1033 units, 66 questions. Verdict: **NOT NEEDED - lift on the paraphrase/miss subset is -2.1 points, below the 10 required**.

Paraphrase / miss subset (16 questions): lexical recall@10 0.198, hybrid 0.177, lift -2.1 points (bar: at least 10). Exact-name loss +0.0 points (bar: at most 1). Overall recall@10 lexical 0.775, hybrid 0.769. Questions without a query vector: 0.

| Category | With gold | Lexical misses | Lexical R@10 | Hybrid R@10 |
|---|---:|---:|---:|---:|
| conflict | 5 | 0 | 1.000 | 1.000 |
| exact_name | 11 | 0 | 1.000 | 1.000 |
| figure | 10 | 0 | 1.000 | 1.000 |
| multi_hop | 8 | 8 | 0.396 | 0.354 |
| paraphrase | 8 | 8 | 0.000 | 0.000 |
| relational | 5 | 0 | 1.000 | 1.000 |
| story | 10 | 0 | 1.000 | 1.000 |

Index size: 528896 bytes as f32 (0.50 MiB). Embedding throughput - and VRAM spill - (not measured: no live embedding run).

#### world 24601: control:gold-biased-128

Model `control:gold-biased-128` (dimension 128), 1033 units, 66 questions. Verdict: **ADOPT (both conditions hold)**.

Paraphrase / miss subset (16 questions): lexical recall@10 0.198, hybrid 0.885, lift +68.8 points (bar: at least 10). Exact-name loss +0.0 points (bar: at most 1). Overall recall@10 lexical 0.775, hybrid 0.968. Questions without a query vector: 0.

| Category | With gold | Lexical misses | Lexical R@10 | Hybrid R@10 |
|---|---:|---:|---:|---:|
| conflict | 5 | 0 | 1.000 | 1.000 |
| exact_name | 11 | 0 | 1.000 | 1.000 |
| figure | 10 | 0 | 1.000 | 1.000 |
| multi_hop | 8 | 8 | 0.396 | 0.771 |
| paraphrase | 8 | 8 | 0.000 | 1.000 |
| relational | 5 | 0 | 1.000 | 1.000 |
| story | 10 | 0 | 1.000 | 1.000 |

Index size: 528896 bytes as f32 (0.50 MiB). Embedding throughput - and VRAM spill - (not measured: no live embedding run).

#### world 9137: stand-in:trigram-hash-128

Model `stand-in:trigram-hash-128` (dimension 128), 1188 units, 64 questions. Verdict: **NOT NEEDED - lift on the paraphrase/miss subset is 4.2 points, below the 10 required; exact-name recall@10 falls 50.0 points, above the 1 allowed**.

Paraphrase / miss subset (16 questions): lexical recall@10 0.198, hybrid 0.240, lift +4.2 points (bar: at least 10). Exact-name loss +50.0 points (bar: at most 1). Overall recall@10 lexical 0.767, hybrid 0.433. Questions without a query vector: 0.

| Category | With gold | Lexical misses | Lexical R@10 | Hybrid R@10 |
|---|---:|---:|---:|---:|
| conflict | 5 | 0 | 1.000 | 0.500 |
| exact_name | 11 | 0 | 1.000 | 0.500 |
| figure | 10 | 0 | 1.000 | 0.000 |
| multi_hop | 8 | 8 | 0.396 | 0.354 |
| paraphrase | 8 | 8 | 0.000 | 0.125 |
| relational | 3 | 0 | 1.000 | 0.667 |
| story | 10 | 0 | 1.000 | 1.000 |

Index size: 608256 bytes as f32 (0.58 MiB). Embedding throughput - and VRAM spill - (not measured: no live embedding run).

#### world 9137: control:random-128

Model `control:random-128` (dimension 128), 1188 units, 64 questions. Verdict: **NOT NEEDED - lift on the paraphrase/miss subset is -6.2 points, below the 10 required**.

Paraphrase / miss subset (16 questions): lexical recall@10 0.198, hybrid 0.135, lift -6.2 points (bar: at least 10). Exact-name loss +0.0 points (bar: at most 1). Overall recall@10 lexical 0.767, hybrid 0.748. Questions without a query vector: 0.

| Category | With gold | Lexical misses | Lexical R@10 | Hybrid R@10 |
|---|---:|---:|---:|---:|
| conflict | 5 | 0 | 1.000 | 1.000 |
| exact_name | 11 | 0 | 1.000 | 1.000 |
| figure | 10 | 0 | 1.000 | 1.000 |
| multi_hop | 8 | 8 | 0.396 | 0.271 |
| paraphrase | 8 | 8 | 0.000 | 0.000 |
| relational | 3 | 0 | 1.000 | 1.000 |
| story | 10 | 0 | 1.000 | 1.000 |

Index size: 608256 bytes as f32 (0.58 MiB). Embedding throughput - and VRAM spill - (not measured: no live embedding run).

#### world 9137: control:gold-biased-128

Model `control:gold-biased-128` (dimension 128), 1188 units, 64 questions. Verdict: **ADOPT (both conditions hold)**.

Paraphrase / miss subset (16 questions): lexical recall@10 0.198, hybrid 0.885, lift +68.8 points (bar: at least 10). Exact-name loss +0.0 points (bar: at most 1). Overall recall@10 lexical 0.767, hybrid 0.967. Questions without a query vector: 0.

| Category | With gold | Lexical misses | Lexical R@10 | Hybrid R@10 |
|---|---:|---:|---:|---:|
| conflict | 5 | 0 | 1.000 | 1.000 |
| exact_name | 11 | 0 | 1.000 | 1.000 |
| figure | 10 | 0 | 1.000 | 1.000 |
| multi_hop | 8 | 8 | 0.396 | 0.771 |
| paraphrase | 8 | 8 | 0.000 | 1.000 |
| relational | 3 | 0 | 1.000 | 1.000 |
| story | 10 | 0 | 1.000 | 1.000 |

Index size: 608256 bytes as f32 (0.58 MiB). Embedding throughput - and VRAM spill - (not measured: no live embedding run).

