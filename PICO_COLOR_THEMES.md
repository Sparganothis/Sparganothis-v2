https://picocss.com/docs/color-schemes

    Examples
    Docs

Getting started

    Quick start
    Version picker New
    Color schemes
    Class-less version
    Conditional styling New
    RTL

Customization

Layout

Content

Forms

Components

About

Getting started
Color schemes

Pico CSS comes with both Light and Dark color schemes, automatically enabled based on user preferences.
Content

    Introduction
    Usage
    Card example

The default color scheme is Light. The Dark scheme is automatically enabled if the user has dark mode enabled prefers-color-scheme: dark;.
Usage
#

Color schemes can be defined for the entire document using <html data-theme="light"> or for specific HTML elements, such as <article data-theme="dark">.

Color schemes at the HTML tag level work great for elements such as a, button, table, input, textarea, select, article, dialog, progress.

CSS variables specific to the color scheme are assigned to every HTML tag. However, we have not enforced specific background and color settings across all HTML tags to maintain transparent backgrounds and ensure colors are inherited from the parent tag.

For some other HTML tags, you might need to explicitly set background-color and color.

section {
  background-color: var(--pico-background-color);
  color: var(--pico-color);
}

Card example
#
Light card
Remember me

<article data-theme="light">
  ...
</article>

Dark card
Remember me

<article data-theme="dark">
  ...
</article>

