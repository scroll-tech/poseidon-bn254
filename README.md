# poseidon-bn254

a computational only implementation of the Poseidon hash using in 
[poseidon-circuit](https://github.com/scroll-tech/poseidon-circuit).

## Comparison with poseidon-circuit

| task                          | poseidon-bn254 cycles | poseidon-circuit cycles |      difference |
|:------------------------------|----------------------:|------------------------:|----------------:|
| hash_with_domain([0, 0], 0) * |                   562 |                   8,199 |    7,467 (-91%) |
| hash_with_domain([1, 2], 3)   |               313,501 |                   8,199 |  1,583 (-19.3%) |
| hash_msg(1, None)             |                 6,580 |                   8,192 |  1,612 (-19.7%) |
| hash_msg(1, Some(1))          |                 6,563 |                   8,192 |  1,629 (-19.9%) |
| hash_msg(10, None)            |                29,605 |                  34,340 |  4,735 (-13.8%) |
| hash_msg(10, Some(10))        |                29,662 |                  34,341 |  4,679 (-13.6%) |
| hash_code("")                 |                   475 |                   8,719 |  8,244 (-94.6%) |
| hash_code(128)                |                20,564 |                  24,329 |  3,765 (-15.5%) |
| hash_code(256)                |                34,393 |                  39,364 |  4,971 (-12.6%) |
| hash_code(1024)               |               117,892 |                 130,052 |  12,160 (-9.4%) |
| hash_code(4096)               |               462,836 |                 505,385 |  42,549 (-8.4%) |
| hash_code(16384)              |             1,831,155 |               1,993,659 | 162,504 (-8.2%) |
| hash_code(24576) **           |             2,743,359 |               2,985,843 | 242,484 (-8.1%) |

&ast;`hash_with_domain([0, 0], 0)` is the hash of the empty message.

** 24576 is the maximum bytecode size limit on ethereum mainnet.