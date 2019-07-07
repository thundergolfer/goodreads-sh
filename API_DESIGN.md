## API Design

#### Preamble

I actually do find gooddreads.com's UI to be too cluttered and poorly set up to support my uses, which are:

1. Track my own reading
2. See what my friends are reading, or have just read

On 1, the UI is not *too* bad. If I'm currently reading something, I can go to the homepage and in the top-left
quickly update my progress in the "Currently Reading" section. It's a little worse with starting a new book. There's 
not "Start a Book" button. Instead I go to search, find the book I've started, and select "Currently Reading". Adding a book
as "Want to Read" is a similar flow. 

On 2, the UI is not good. The middle of the homepage is the newsfeed, but even with my only **3**
friends I find it packed with their "Wants to Read" updates, or with new friends they have. 
This information is irrelevant for purpose 2, and I'd like to filter it. 

More generally, I'm not a fan of the "2019 READING CHALLENGE", "RECOMMENDATIONS", "NEWS & INTERVIEWS", and "GOODREADS CHOICE AWARDS"
sections that fill the rest of the screen. I personally don't find them that useful and so I'd prefer to 
strip them away and focus on 1 and 2. I'd love for the homepage to show me metrics and graphs of my reading. That'd be cool.

#### Design

First let's tackle **1.A: Updating progress on an existing book** 

The book will already exist in my "Currently Reading" shelf, so I could do something like this: 

`goodreads-sh update`

and have it list out the "Currently Reading" shelf like so: 

```bash
1. <Book Title, Author> - <Progress>
2. <Book Title, Author> - <Progress>
3. <Book Title, Author> - <Progress>
4. <Book Title, Author> - <Progress>
5. <Book Title, Author> - <Progress>
...
```

Then it could prompt for a selection from the numbered list. I would provide one, and then the CLI
would prompt me for a progress update. It should be able to parse either a page number or a percentage.

I think we can improve on this though. As I know what books I'm currently reading I should be able to do this:

`goodreads-sh update --book "<book title>" --progress <page num>`

Even without the user providing a title string that exactly matches what's in currently reading. A matching algorithm
like edit-distance or longest-common-sequence should be able to easily get the right book.

-----

**1.B: Starting a new book**

For this one the CLI will need to receive some kind of book identifying input from the user and then
do a search of the goodreads.com backend to find the matching book. Something like this:

`goodreads-sh new --book "<book title>" --progress <page num OR percentage>`

Then the CLI will search and present the user matches like so:

```bash
1. <Book Title, Author> - <Progress>
2. <Book Title, Author> - <Progress>
...
```

Then it could prompt for a selection from the numbered list and execute the update, or if the search failed
prompt the user to try again.

> ❗️: An alternative subcommand name could be "start"

-----

**2. See what friends are reading**

\*will write this up later\* 