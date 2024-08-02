https://keepachangelog.com/en/1.1.0/


## CHANGELOG syntax
```txt
header: multiline_text | _

release_title: ## [*] | ## [*] - *

release_section_title: ### *

release_header: multiline_text | _

release_footer: multiline_text| _

release_note: - *

release_section: release_section_title release_note+*

release: release_title release_header release_section* release_footer

footer_link: [*]: *

changelog: header release* footer_link*
```