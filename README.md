# term-ai

## NOTE

The AI responses shown below are subject to change.

## What is it?

This program calls AI(ChatGPT) from the terminal and enables interactive conversation.

## How to use

### 1. Set environment

You must set the following environment variables.

```bash
export OPENAI_API_KEY=<your api key>
```

### 2. Call TermAI Command

```bash
$ termai chat
```

## Features(SubCommands)

### Chat

You can have a conversation with GPT3 or GPT4 at your terminal.

```bash
$ termai chat -v 4

YOUR_NAME> Hello, I'm <your name>. I'm an English teacher.
gpt> Hello, I'm GPT. I'm an AI.
```

You can specify below options

```
-v,--gpt-version <VERSION>
-c,--code-capture
-r,--code-reviewer
-t,--translator <TO_LANG>
-f,--file-translator <TO_LANG>
-s,--speaker

```

### Code capture

If gpt response contain code, then sample_xxx file is created with captured code.

```bash
$ termai cc "Please write calculate fibonacci sequence program by python"

OK! so code is below.

python
...

```

You can specify below options

```
-v,--gpt-version <VERSION>
```

### Code review

You can easily have your code reviewed by AI.

If you want a code review, simply input the path to the code and the AI(GPT) will conduct the review.

```bash
$ termai cr CODE_PATH

Your code is so good!
...

```

You can specify below options

```
-v,--gpt-version <VERSION>
```

## Speaker

You can speak gpt response.

NOTE: Speaker is text-to-speech feature using the `say` command.So,this feature is only available on macOS as it depends on that specific command.

```bash
$ termai speaker "Hello GPT!"

Hello,How are you?

```

You can specify below options

```
-v,--gpt-version <VERSION>
```

### Translator

You can translate your message by AI(GPT).

```bash
$ termai ten "こんにちは GPT!"

Hello GPT!

$ termai tjp "Hello GPT!"

こんにちは GPT

$ termai tko "Hello GPT!"

```

You can specify below options

```
-v,--gpt-version <VERSION>
-f,--file-path <FILE_PATH>
```

