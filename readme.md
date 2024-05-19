# Blog On It 
<a  href="https://youtu.be/k-t4vqd534Y?si=psi6ZEypAcOF8pdw"><img src="https://i3.ytimg.com/vi/k-t4vqd534Y/maxresdefault.jpg" target="_blank" rel="noopener"  height=200px></a>


#### Planned Idea
- [x] Converts MD files into basic html files
- [x] Generates a table of contents as home page
- [x] Follows Obsidian style: file name implied for h1, others are lower level headers.

Includes bad rust habits. Don't expect me to use `Rc` or `RefCell`. I have two brain cells that do not know of each other's existence.

## How to Use?
tl;dr: `cargo run <folder containing markdown files> <website name>`
- Make Markdown files. Math mode and codeblocks (highlighting, parsing is supported) are not supported as of now. And by extension, none of the advanced funny stuff.
- Organize them into folders/directories (call it whatever you want you windows/unix purists).
- (Optional) Add an index.md with raw html. This html would be injected into the body before the list of contents in the front page (index.html).
- List of Contents will have a descending order according to it's last modified order (hopefully). 
- Run `cargo run <markdown folder> <website name>`
- Once built, you can also use the binary in `target` folder/directory. 
- The website will be generated in a folder/directory called `website`.
- Style it yourself. All have styles.css in the `<head>`, if you are into that stuff.

Upload somewhere and enjoy.

#### Example
Example of generated files in examples folder. In my defense, I provided only the topic. ChatGPT wrote the blogs.

It Just Works<sup>TM</sup>  

<img src="https://yt3.ggpht.com/a/AATXAJxuZBNfke48M_7TcSsN9iMtJmaE1JTNVVfEeg=s900-c-k-c0xffffffff-no-rj-mo" target="_blank" rel="noopener"  height=100px >
<sup>Todd's Mark of Approval (not real)</sup>
