# splits'r'us

Splits a string on whitespace returning order-preserving combinations of 1, 2 and 3 words.

Uses unstable rust features so likely to break in future.

e.g.

```python
splits_rs.split_string("Sion Softside Expandable\nRoller Luggage, Black")
```

```python
[
    'Sion Softside Expandable',
    'Softside Expandable Roller',
    'Expandable Roller Luggage,',
    'Roller Luggage, Black',
    'Sion Softside',
    'Softside Expandable',
    'Expandable Roller',
    'Roller Luggage,',
    'Luggage, Black',
    'Sion',
    'Softside',
    'Expandable',
    'Roller',
    'Luggage,',
    'Black',
]
```

Also provides a version that accepts a numpy array `splits_rs.split_strings()` or `pyarrow` arrays with `splits_rs.split_strings_arrow()`

The following example shows the arrow method as it is 10x faster.

```python
import pandas as pd
import pyarrow as pa
import splits_rs
import re
from pathlib import Path

amazon = pd.read_csv(
    "../data/input/amazon_products.csv.zip",
).set_index("asin")["title"]
amazon = amazon.dropna()
assert amazon.isna().sum() == 0
amazon = amazon.str.replace("[\r\n]", " ", regex=True, flags=re.DOTALL|re.MULTILINE)
amazon = amazon.str.replace("[^A-Za-z0-9]", " ", regex=True, flags=re.DOTALL|re.MULTILINE)
# 9 seconds

# len(amazon) = 1426336 rows
# amazon.memory_usage(deep=True) = 345 Megabytes

# convert to pyarrow
arr = pa.array(amazon)
# 0.1 seconds

(a,b) = splits_rs.split_strings_arrow(arr)
# 7 seconds 🤯

result = pd.concat([a.to_pandas(),b.to_pandas()], axis='columns')
# 13 seconds

result
```

||index|token|
|--|--|--|
|0|0|Sion Softside Expandable|
|1|0|Softside Expandable Roller|
|2|0|Expandable Roller Luggage|
|3|0|Roller Luggage Black|
|4|0|Luggage Black Checked|
|...|...|...|
|25|0|29|
|26|0|Inch|
|27|1|Luggage Sets Expandable|
|28|1|Sets Expandable PC|
|29|1|Expandable PC ABS|
|...|...|...|

> 80278819 rows × 2 columns


### Performance

Switching to `pyarrow` as the return type made a huuuge difference to speed. The downside is that `rayon` does not seem to be easy to use with the `pyarrow` rust crate.

Maybe the `numpy` solution can be made to run a lot faster if we can return `numpy` arrays instead of python types.
