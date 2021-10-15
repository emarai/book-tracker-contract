# How to Compile

```
$ yarn build:contract
```

# How to deploy on testnet

```
$ near dev-deploy
```

# Call Functions

## Add book

### Book interface
```
pub struct Book {
    title: String,
    description: String,
    status: Status,
    image: String,
}
```

## Add book call function
```
add_book '{"book":{"description":"Tutorial for mechanics","image":"https://example.com","title":"Motorcycle Mechanics 101","status":"List"}}'

return book_id
```

## Update book

```
update_book '{"book_id":"1", "status":"Read"}'
```

## Delete book

```
delete_book '{"book_id":"1"}'
```

# View methods

## Get book

```
get_book '{"book_id":"1"}'
```

## Get books

Get all books from gnaor.testnet
```
get_books '{"account_id":"gnaor.testnet", "skip":0, "limit": 10}'
```

Get all books 
```
get_books '{"skip":0, "limit": 10}'
```
