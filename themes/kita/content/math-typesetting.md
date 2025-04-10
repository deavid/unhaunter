+++
title = "Math Typesetting"
date = "2022-10-20"
description = "A brief guide to setup KaTeX"
extra.math = true
+++

Kita theme support $\LaTeX$ mathematical formulas using [KaTeX](https://katex.org/).

<!--more-->

- To enable KaTeX globally, set the parameter `extra.math` to `true` in a project's configuration.
- To enable KaTeX on a per page basis, include the parameter `extra.math = true` in the frontmatter of content files.

**Note:** Use the online reference of [Supported TeX Functions](https://katex.org/docs/supported.html).

### Examples

#### Inline math

```markdown
When $x = \pi$, Euler's formula may be rewritten as $e^{i \pi} + 1 = 0$.
```

When $x = \pi$, Euler's formula may be rewritten as $e^{i \pi} + 1 = 0$.

#### Block math

```markdown
$$
 \varphi = 1+\frac{1} {1+\frac{1} {1+\frac{1} {1+\cdots} } }
$$
```

$$
 \varphi = 1+\frac{1} {1+\frac{1} {1+\frac{1} {1+\cdots} } }
$$
