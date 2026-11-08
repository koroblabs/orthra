; Where prose lives in XML. Captures: @comment, @prose, @string, @value, @skip (`tree_sitter_host.rs`).
; What is not captured is not checked: a tag or attribute name, a namespace, a processing
; instruction, a doctype and a CDATA section (`<![CDATA[ … ]]>` is script, SQL or markup that
; is somebody's other language).

(Comment) @comment

; Text between tags. The risk in XML is the element whose text is a value: `pom.xml`'s
; `<artifactId>maven-compiler-plugin</artifactId>`, `<TargetFramework>net8.0</TargetFramework>`,
; a plist's `<key>CFBundleName</key>`. Such a value is the only thing in its element and has no
; space in it, so it is held to known typos (`@value`). Text with a space in it is a sentence, and
; so is a word that sits beside other markup (`Use <b>this</b> one`).
((CharData) @prose
  (#match? @prose "\\S\\s+\\S"))
(content (CharData) @prose . (_))
(content (_) . (CharData) @prose)
((content . (CharData) @value .)
  (#match? @value "^\\s*\\S+\\s*$"))

; Code or a placeholder in the middle of a sentence: DocBook's inline code, Android's `<xliff:g>`.
((element (STag (Name) @_tag)) @skip
  (#match? @_tag "^(literal|code|command|filename|option|varname|classname|methodname|function|parameter|replaceable|envar|systemitem|userinput|computeroutput|programlisting|screen|uri|package|prompt|constant|property|tag|sgmltag|markup|type|database|application|keycap|keycombo|menuchoice|guilabel|guibutton|guimenu|guimenuitem|xliff:g|script|style)$"))

; People: Maven's `<developer>` and `<contributor>`, a `.nuspec`'s `<authors>`. Their names, firms
; and roles are not words, and are written where the manifest declares them.
((element (STag (Name) @_who)) @skip
  (#match? @_who "^(developer|contributor|developers|contributors|author|authors|owners|copyright|organization)$"))

; The attributes whose value is text for a reader: Android's `android:text`, `android:hint`,
; `android:contentDescription`, `android:label`, XAML's `Text`, `Content`, `Header`, `ToolTip`,
; and the XML-wide `title`, `alt`, `description`, `summary`, `label`.
((Attribute
   (Name) @_name
   (AttValue) @string)
  (#match? @_name "^([A-Za-z0-9_]+:)?(text|title|label|hint|summary|description|alt|tooltip|contentDescription|prompt|subtitle|message|Text|Title|Content|Header|ToolTip|Description|Label|Caption|Summary)$"))
