# Contents

- [gen Command](#gen)
- [fullgen Command](#fullgen)
- [tofile Command](#tofile)
- [tofile_full Command](#tofile_full)
- [english_gen Command](#english_gen)
- [your_list Command](#your_list)
- [english_to_file Command](#english_to_file)

## Gen

The `gen` command will return a random string which uses
only the normal characters. It returs as a `String` type.

```rust
thorne::gen(/* char_nr: */ 10, /* times: */ 10);
```

## Fullgen

The `fullgen` command is identical to the `gen` command but
it uses every char possible.

```rust
thorne::fullgen(/* char_nr: */ 10, /* times: */ 10)
```

## ToFile

The `tofile` command will write to a file the generated string.

- tofile uses the normal list

```rust
thorne::tofile(/* char_nr: */ 10, /* times: */ 10, String::from("filename"))
```

## ToFile_full

The `ToFile_full` command will write to a file the generated string.

- tofile uses the full list

```rust
thorne::tofile_full(/* char_nr: */ 10, /* times: */ 10, String::from("filename"))
```

## English_Gen

The `english_gen` command will generate a random english word

``` rust
thorne::english_gen(/* char_nr: */ 10, /* times: */ 10)
```

## Your_list

The `your_list` command will generate random string from a
list you written

```rust
thorne::your_list( /* list: */ &mut ["go", "rust", "c++", "c"], /* char_nr: */ 3, /* test: */ 1));
```

## english_to_file

The `english_to_file` command will generate a number of
english words to a file.

> NOTE: `english_to_file` is very slow and unoptimised

```rust
thorne::english_to_file((/* char_nr: */ 10, /* times: */ 10, String::from("filename"))
```
