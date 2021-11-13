# petridish

<div align="center">
  <a href="https://github.com/petridish-dev/petridish/actions">
    <img alt="CI" src="https://github.com/petridish-dev/petridish/actions/workflows/ci.yml/badge.svg">
  </a>
  <a href="https://crates.io/crates/petridish">
    <img alt="Version info" src="https://img.shields.io/crates/v/petridish?colorB=319e8c">
  </a>
  <a href="https://github.com/zen-xu/job-book/blob/master/LICENSE">
    <img alt="GitHub" src="https://img.shields.io/github/license/petridish-dev/petridish">
  </a>
  <br>
  A command-line utility that creates project structure.
</div>


If you have heard of the [`cookiecutter`](https://github.com/cookiecutter/cookiecutter) project, `petridish` is a rust implementation of it.


## Features

| Feature                         | Description                                                                                            |
| ------------------------------- | ------------------------------------------------------------------------------------------------------ |
| **Cross platform**              | Windows, Linux, MacOS                                                                                  |
| **More flexible configuration** | `petridish` use yaml file to define the template variables                                             |
| **More humanized prompt**       | support different kinds of prompt (`input`, `choice`, `multi choices`, `confirm`)                      |
| **Powerful template engine**    | we use [`tera`](https://github.com/Keats/tera) as our template engine, which is based on Jinja2/Django |
| **One binary**                  | one binary run everywhere                                                                              |

## Usage

Like [`cookiecutter`](https://github.com/cookiecutter/cookiecutter), you should provide directory structure like this:

![](assets/petridish-structure.png)

Let's have a look at the `petridish.yaml` file:

```yaml
prompts:
  - name: name                   # normal input prompt
    message: what's your name

  - name: age                    # single choice prompt
    choices: [10, 20, 30]
    default: 20

  - name: hobby                  # multi choices prompt
    choices: [running, swimming]
    multi: true

  - name: is_geek                # confirm prompt
    confirm: true

entry_dir: "{{ repo_name }}"     # default is {{ repo_name }}
entry_dir_prompt_message: repo dir name?
```

| Prompt kind     | Field       | Description                             | optional |
| --------------- | ----------- | --------------------------------------- | :------: |
| `normal input`  | **name**    | template_var_name                       |          |
|                 | **message** | prompt message                          |    ✅     |
|                 | **default** | default value                           |    ✅     |
| `single choice` | **name**    | template_var_name                       |          |
|                 | **message** | prompt message                          |    ✅     |
|                 | **choices** | choice items                            |          |
|                 | **default** | default value                           |    ✅     |
| `multi choice`  | **name**    | template_var_name                       |          |
|                 | **message** | prompt message                          |    ✅     |
|                 | **choices** | choice items                            |          |
|                 | **default** | default values                          |    ✅     |
|                 | **multi**   | must be `true` or `default` is provided |          |
| `confirm`       | **name**    | template_var_name                       |          |
|                 | **message** | prompt message                          |    ✅     |
|                 | **default** | default value (default false)           |    ✅     |


## Template

A Tera template is just a text file where variables and expressions get replaced with values when it is rendered. The syntax is based on Jinja2 and Django templates.

There are 3 kinds of delimiters and those cannot be changed:

- {{ and }} for expressions
- {% and %} for statements
- {# and #} for comments

More syntax details can be found in [`tera`](https://tera.netlify.app/docs/#templates).
