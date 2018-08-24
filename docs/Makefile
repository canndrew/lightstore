MARKDOWN = pandoc --from markdown --to html --standalone --css pandoc.css --mathml --lua-filter=links-to-html.lua
all: $(patsubst %.md,%.html,$(wildcard doc/*.md)) index.html Makefile pandoc.css

clean:
	rm -f index.html
	rm -f $(patsubst %.md,%.html,$(wildcard doc/*.md))

%.html: %.md pandoc.css
	$(MARKDOWN) $< --output $@

