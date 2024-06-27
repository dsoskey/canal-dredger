# Scryfall migration notes

## known delete subsets

These migrations are deletes without metadata where we know the set of cards but not the `oldId` -> `name` mapping. They must be manually added to as we encounter broken Cubecobra history entries.

- [mh2 rare prerelease promos](./mh2-migrations.json) ([scryfall search](https://scryfall.com/search?q=set%3Amh2+-in%3Apmh2+r%E2%89%A5r+is%3Areprint+not%3Afetchland+-border%3Aborderless+not%3Aold&unique=cards&as=grid&order=name))