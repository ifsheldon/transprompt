# transprompt

Prompt-centric framework for developing LLM applications in Rust

**Note: `transprompt` is not stable yet, so the APIs are subject to change.**

## Usage

To use it, add a dependency in `Cargo.toml`

```toml
transprompt = "0.14"
```

## Documentation

Run the below command to build the documentation and open it in the browser.

```shell
cargo doc --open
```

## Why `transprompt`

Because I'm done with layers of object-oriented abstraction that are mixed with inheritance hierarchies and methods that
are overloaded and overridden from nowhere.

LLM programming, a fancy name of prompt engineering, starts with prompts, so it should be prompt-centric (or
data-driven if you come from software engineering).

## Concepts and Design

The overall of `transprompt` follows data-driven design. The APIs are designed to be as explicit as possible, so users
should easily track every step that composes a prompt. The API hierarchy also aims to be as flat as possible. Cycle
speed is NOT a top priority since LLM can take trillions of cycles to respond to a request.

### Prompt Template and Placeholder

As straightforward as its name, it's a template of prompts.

For example, a template looks like

```text
You are a friendly and helpful assistant. Today is {{date}}.
```

Now, `{{date}}` is a placeholder, a slot to be filled, in this template, which has a name `"date"`.

The format of a named placeholder is simply `{{whatever name you like}}`. The name can have any strings except those
containing line breaks `"\n"`and `"\r\n"`.
> Why in this format?
>
> Because of KISS and my limited regex proficiency.

### Partial Prompt

While a prompt template is a blueprint, a partial prompt is an incomplete construction of the template, which means it
has empty slots (AKA placeholders).

A `PartialPrompt` comes only from `PromptTemplate::construct_prompt`.

A `PartialPrompt` records which placeholder gets filled by what value and also unfilled placeholders.

When all placeholders in a `PartialPrompt` are filled, it's complete and thus ready to be transformed into a concrete
prompt. This is simply done via `PartialPrompt::complete`.

### Filler

Anything that fills one or more placeholders in a partial prompt.

In Rust, it means anything that implements `FillPlaceholders` and at least one of `Fill`, `FillMut`, `FillWith<CTX>`
and `FillWithMut<CTX>`.

Fillers fill placeholders. Placeholders get filled via `PartialPrompt::fill` or `PartialPrompt::try_fill`.

> A simple example is a date filler, which fills a placeholder name `date` that is represented in a template
> as `{{date}}`.

A filler can also be a composition of many fillers. Therefore, in a complex workflow, a `PartialPrompt` can be filled by
concurrent fillers in multiple stages.

### Endpoint or LLM

The endpoint of `PromptTemplate -> PartialPrompt -> complete prompt (a String)` pipeline is LLM, which consumes a prompt
and produces a reply.

You can do any post-processing on the reply, but we will leave that in utilities.

Or, you can even kick off another pipeline that transforms a prompt template with fillers, so then the endpoint is a new
start!

### Application or Agent or Whatever

A LLM application is just a ordered collection of:

* Prompt templates
* Fillers (and intermediate partial prompts)
* Post-processing stages

## TODOs

Sorted from top to down by importance:

- [x] Vector database connection: simplest Qdrant DB for now
- [x] LLM integration: basics for OpenAI ChatGPT
  - [ ] Other LLM support
- [x] Documentation: basic documentation for now
- [ ] ~~Integration of [guidance](https://github.com/microsoft/guidance)~~
    - I don't know how to do it yet, but the library is fxxking genius, despite its algorithmic simplicity.
    - No emergent need because of OpenAI function calling
- [ ] Utilities including
    - [x] Simple JSON postprocessing: Only extracting out valid JSON content from a string for now
      - [ ] ~~Add Support for Jsonformer~~ No emergent need because of OpenAI function calling
    - [ ] Frequently used applications/agents
      - [ ] Generative Agents
    - [x] Token counting utils: Now only basic tiktoken support
- [ ] Examples
- [ ] Future engineering improvements like advance compile time checking or type system dance
- [ ] Python counterpart?
    - I love Python's dynamism just like I like Rust's stasis, so I would love to see a prompt-centric counterpart in
      Python.
    - It seems Semantic Kernel is similar?

## Contribution

Contributions are always welcome. Please see TODOs.

## License

`transprompt` will always remain free under Apache license.

## Attribution

* `async_openai`: The codebase of `transprompt` has copied content from this crate, which
  is `transprompt::utils::llm::openai::ConversationConfig`.
* `tiktoken-rs`: In `transprompt::utils::token::tiktoken`, we re-export the `tiktoken-rs` crate.
