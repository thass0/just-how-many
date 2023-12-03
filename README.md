# Just How Many?

Just How Many? is a web service that tracks the number of visitors a web page has. I uses **0** JavaScript, and has the reader's privacy in mind.

The way this works is based on a simple CSS trick. This trick was first used by [Herman Martinus for Bear](https://herman.bearblog.dev/how-bear-does-analytics-with-css/).

``` css
body:hover {
	<!-- This will trigger a get when the reader hovers with their cursor over the body tag. -->
	border-image: url("http://just-how-many.com/hit/66c22d77-265e-4dbb-aadf-c57b46cb187b");
    border-width: 0;
}
```

By using the hover property, we also get bot-protection for free!

The CLI binary (`jhm`) can be used to register pages and check the number of hits a page has.
