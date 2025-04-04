+++
title = "Shortcodes"
date = "2022-10-20"
description = "The Kita theme shortcodes."
[taxonomies]
tags = ["markdown", "css", "html"]
[extra]
mermaid = true
+++

The Kita theme providers multiple shortcodes.

Never heard of shortcodes? See [Zola documentation](https://www.getzola.org/documentation/content/shortcodes/) for more information.

## Mermaid

To use Mermaid in your page, you have to set `extra.mermaid = true` in the frontmatter of page.

```markdown
+++
title = "Your page title"

[extra]
mermaid = true
+++
```

Then you can use the `mermaid()` shortcodes like:

```markdown
{%/* mermaid() */%}

graph TD;
A-->B;
A-->C;
B-->D;
C-->D;

{%/* end */%}
```

This will be rendered as:

{% mermaid() %}

graph TD;
A-->B;
A-->C;
B-->D;
C-->D;

{% end %}

In addition, you can use code block inside `mermaid()` shortcodes and the code block will be ignored.

The code block prevents formatter from breaking mermaid's formatting.

````markdown
{%/* mermaid() */%}

```mermaid
sequenceDiagram
    participant Alice
    participant Bob
    Alice->>John: Hello John, how are you?
    loop Healthcheck
        John->>John: Fight against hypochondria
    end
    Note right of John: Rational thoughts <br/>prevail!
    John-->>Alice: Great!
    John->>Bob: How about you?
    Bob-->>John: Jolly good!
```

{%/* end */%}
````

This will be rendered as:

{% mermaid() %}

```mermaid
sequenceDiagram
    participant Alice
    participant Bob
    Alice->>John: Hello John, how are you?
    loop Healthcheck
        John->>John: Fight against hypochondria
    end
    Note right of John: Rational thoughts <br/>prevail!
    John-->>Alice: Great!
    John->>Bob: How about you?
    Bob-->>John: Jolly good!
```

{% end %}

## Admonition

The `admonition()` shortcode displays a banner to help you put notice in your page.

You can use the `admonition()` shortcode like:

```markdown
{%/* admonition(type="tip", title="tip") */%}
The `tip` admonition.
{%/* end */%}
```

The admonition shortcode has 12 different types:

{% admonition(type="note", title="note") %}
The `note` admonition.
{% end %}

{% admonition(type="abstract", title="abstract") %}
The `abstract` admonition.
{% end %}

{% admonition(type="info", title="info") %}
The `info` admonition.
{% end %}

{% admonition(type="tip", title="tip") %}
The `tip` admonition.
{% end %}

{% admonition(type="success", title="success") %}
The `success` admonition.
{% end %}

{% admonition(type="question", title="question") %}
The `question` admonition.
{% end %}

{% admonition(type="warning", title="warning") %}
The `warning` admonition.
{% end %}

{% admonition(type="failure", title="failure") %}
The `failure` admonition.
{% end %}

{% admonition(type="abstract", title="danger") %}
The `danger` admonition.
{% end %}

{% admonition(type="bug", title="bug") %}
The `bug` admonition.
{% end %}

{% admonition(type="example", title="example") %}
The `example` admonition.
{% end %}

{% admonition(type="quote", title="quote") %}
The `quote` admonition.
{% end %}

## Gallery

The `gallery()` shortcode is very simple html-only clickable picture gallery that displays all images from the page assets.

It's from [Zola documentation](https://www.getzola.org/documentation/content/image-processing/)

```markdown
{{/* gallery() */}}
```

{{ gallery() }}
