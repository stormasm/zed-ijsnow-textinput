### Initial work by Isaac Snow

- [ijsnow](https://github.com/ijsnow)
- [Zed PR 9386](https://github.com/zed-industries/zed/pull/9386)

Release Notes:

Adds an element to gpui that allows editing the data in a Model<String>
Test with the editable_text example.

```rust
cargo run -p gpui --example editable_text
```

There is definitely some fine tuning to be done, but I wanted to put a draft up to see if there were any comments on the design or direction. Everything is pretty basic but it seems to work for just basic string manipulation. Here are some known problems:

- Mouse down doesn't get reset when mouse up happens outside the text (should be a quick fix)
- Multi line text data can be passed to the element in single line mode and not much changes
- Need to do select down/select up for multi line. I forgot about it
- Multiline support in general is a bit off
- If a line contains only a newline, you cant move the cursor to it by clicking it. The index_for_position code needs to be modified a bit more
- Much more I'm sure. I just kind of brute forced the design. For example it would probably make sense to accept a view that implements ViewInputHandler as well as just a Model<String> with a custom InputHandler.
- I'm sure there are some optimizations that could be done.

### References

- [fork of ijsnow zed with text-input branch ](https://github.com/stormblog/zed/tree/text-input)
