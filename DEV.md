https://keepachangelog.com/en/1.1.0/


## CHANGELOG syntax
```txt
header: multiline_text | _

release_title: ## [text] | ## [text] - text

release_section_title: ### text

release_header: multiline_text | _

release_footer: multiline_text| _

release_note: - multiline_text

release_section: release_section_title release_note+*

release: release_title release_header release_section* release_footer

footer_link: [text]: text

changelog: header release* footer_link*
```



can you modify this regex to match: ## [*] | ## [*] - *

This is the regex:
## \[.*\]
## \[.*\](?: - .*)?

## \[.*\](?: - .*)?

