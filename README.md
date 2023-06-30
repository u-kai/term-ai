# term-ai

## NOTE

The AI responses shown below are subject to change.

---

## What is it?

This program calls AI(ChatGPT) from the terminal and enables interactive conversation.

---

## How to use

### Set environment

You must set the following environment variables.

```bash
export OPENAI_API_KEY=<your api key>
```

---

## GPT3

You can have a conversation with AI(GPT3).

```bash
termai gpt3

YOUR_NAME> Hello, I'm <your name>. I'm an English teacher.
gpt> Hello, I'm GPT. I'm an AI.
```

---

## Code capture

You can easily capture the code generated by the AI(GPT3).

````bash
termai capt

YOUR_NAME>足し算をおこなうpythonのコードを教えてください．
gpt> pythonで足し算をするコードは以下です.

```python
def add(a, b):
    return a + b
```

...
````

The code displayed terminal is also saved in the sample-for-gpt-RAND.LANG.

Here, RAND is a random number and LANG is the language of the code.

---

## Code review

You can easily have your code reviewed by AI.

If you want a code review, simply input the path to the code and the AI(GPT3) will conduct the review.

Otherwise, you can type anything else for a normal conversation.

```bash
termai review

YOUR_NAME> YOUR_CODE_PATH
gpt> あなたのコードは...
YOUR_NAME>なるほどね．ありがとう
gpt> どういたしまして!!...

```

---

## Speaker

Speaker is text-to-speech feature using the `say` command.So,this feature is only available on macOS as it depends on that specific command.

```bash
termai speaker

YOUR_NAME> hello 今日はいい天気ですね．
gpt> hello. I'm GPT. 今日はいい天気なんですか？AIだからわからないです...

```

---

## English Teacher

---

## Command Option

```bash
--your-display, -y
--ai-display, -a
--help, -h
```
