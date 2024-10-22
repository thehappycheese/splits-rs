# splits'r'us

Splits a string on whitespace (any length/type)

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

also provides a version that accepts a numpy array `splits_rs.split_strings()`

```python
amazon = pd.read_csv(
    "../data/input/amazon_products.csv.zip",
    #nrows=10
).set_index("asin")["title"]#.astype(pd.ArrowDtype(pa.string()))
amazon = amazon.dropna()
assert amazon.isna().sum() == 0
amazon = amazon.str.replace("[\r\n]", " ", regex=True, flags=re.DOTALL|re.MULTILINE)
amazon = amazon.str.replace("[^A-Za-z0-9]", " ", regex=True, flags=re.DOTALL|re.MULTILINE)


tokens = pd.DataFrame(
    splits_rs.split_strings(amazon.values),
    columns=["index","token"]
)
tokens

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

### Performance

Currently the approach used is not great. Although string processing is run in parallel and completes very fast, the handover between rust, python and pandas is very slow and takes most of the time.

We have to deal with python GIL nonsense because numpy stores each string as a python object. This prevents us from reading the strings in-place from multiple threads, even though this would be perfectly safe.

Maybe could be a lot faster if we could receive and return Arrow arrays?
