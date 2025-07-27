// markdownRenderer.ts - Enhanced markdown renderer with proper parsing order
export class MarkdownRenderer {
  static render(text: string): string {
    if (!text) return ''
    
    console.log('=== MARKDOWN RENDER START ===')
    console.log('Input length:', text.length)
    
    // Create a copy to work with
    let processed = text.trim()
    
    // Store code blocks to protect them from other processing
    const codeBlocks: string[] = []
    const inlineCodes: string[] = []
    const codeBlockPlaceholder = (index: number) => `⏿⏿⏿CODEBLOCK${index}⏿⏿⏿`
    const inlineCodePlaceholder = (index: number) => `⏿⏿⏿INLINECODE${index}⏿⏿⏿`
    
    // Extract and protect code blocks first
    processed = processed.replace(/```(\w+)?\n?([\s\S]*?)```/g, (match, lang, code) => {
      const index = codeBlocks.length
      const language = (lang || 'text').toLowerCase()
      const cleanCode = code.trim()
      
      const highlightedCode = this.highlightCode(cleanCode, language)
      const escapedCleanCode = cleanCode.replace(/`/g, '\\`').replace(/\$/g, '\\$')
      
      const codeBlockHtml = `<div class="code-block bg-black/40 border border-white/20 rounded-lg p-4 my-4 font-mono text-sm overflow-x-auto">
        <div class="code-header flex items-center justify-between mb-3">
          <div class="text-xs text-white/60 uppercase tracking-wide font-semibold">${language}</div>
          <button onclick="navigator.clipboard.writeText('${escapedCleanCode}')" 
                  class="text-xs text-white/40 hover:text-white/70 transition-colors px-2 py-1 rounded hover:bg-white/10">
            Copy
          </button>
        </div>
        <div class="code-content leading-relaxed">${highlightedCode}</div>
      </div>`
      
      codeBlocks.push(codeBlockHtml)
      return codeBlockPlaceholder(index)
    })
    
    // Store inline code to protect it
    processed = processed.replace(/`([^`\n]+)`/g, (match, code) => {
      const index = inlineCodes.length
      const inlineCodeHtml = `<code class="bg-black/40 px-1.5 py-0.5 rounded text-sm font-mono text-cyan-300">${this.escapeHtml(code)}</code>`
      inlineCodes.push(inlineCodeHtml)
      return inlineCodePlaceholder(index)
    })
    
    // Process block-level elements first
    processed = this.processBlockElements(processed)
    
    // Process inline elements
    processed = this.processInlineElements(processed)
    
    // Restore code blocks and inline code
    console.log('Restoring code blocks:', codeBlocks.length)
    codeBlocks.forEach((html, index) => {
      const placeholder = codeBlockPlaceholder(index)
      console.log('Replacing placeholder:', placeholder, 'with HTML length:', html.length)
      processed = processed.replace(new RegExp(placeholder, 'g'), html)
    })
    
    console.log('Restoring inline codes:', inlineCodes.length)
    inlineCodes.forEach((html, index) => {
      const placeholder = inlineCodePlaceholder(index)
      processed = processed.replace(new RegExp(placeholder, 'g'), html)
    })
    
    // Add raw markdown display for testing (commented out)
    // const debugSection = `
    //   <div class="markdown-debug border-t border-white/20 mt-6 pt-4">
    //     <div class="text-xs text-white/50 font-semibold mb-2">RAW MARKDOWN:</div>
    //     <pre class="bg-gray-800/50 border border-white/10 rounded p-3 text-xs text-white/70 overflow-x-auto font-mono whitespace-pre-wrap">${this.escapeHtml(text)}</pre>
    //   </div>
    // `
    
    console.log('=== MARKDOWN RENDER END ===')
    // console.log('Final output length:', (processed + debugSection).length)
    
    // return processed + debugSection
    return processed
  }
  
  private static processBlockElements(text: string): string {
    const lines = text.split('\n')
    const result: string[] = []
    let i = 0
    
    while (i < lines.length) {
      const line = lines[i]
      const trimmed = line.trim()
      
      // Skip empty lines
      if (!trimmed) {
        result.push('')
        i++
        continue
      }
      
      // Headers
      if (trimmed.startsWith('# ')) {
        result.push(`<h1 class="text-2xl font-bold text-white mt-6 mb-4">${trimmed.slice(2)}</h1>`)
        i++
        continue
      }
      if (trimmed.startsWith('## ')) {
        result.push(`<h2 class="text-xl font-semibold text-white/95 mt-5 mb-3">${trimmed.slice(3)}</h2>`)
        i++
        continue
      }
      if (trimmed.startsWith('### ')) {
        result.push(`<h3 class="text-lg font-semibold text-white/90 mt-4 mb-2">${trimmed.slice(4)}</h3>`)
        i++
        continue
      }
      
      // Blockquotes
      if (trimmed.startsWith('> ')) {
        const blockquoteLines = []
        while (i < lines.length && lines[i].trim().startsWith('> ')) {
          blockquoteLines.push(lines[i].trim().slice(2))
          i++
        }
        result.push(`<blockquote class="border-l-4 border-blue-500/50 pl-4 italic text-white/80 my-3 bg-blue-500/5 py-2">${blockquoteLines.join('<br>')}</blockquote>`)
        continue
      }
      
      // Horizontal rules
      if (trimmed === '---' || trimmed === '***' || trimmed === '___') {
        result.push('<hr class="border-white/20 my-6" />')
        i++
        continue
      }
      
      // Unordered lists (handle empty lines between items)
      if (/^[\*\-\+]\s/.test(trimmed)) {
        const listItems = []
        
        while (i < lines.length) {
          const currentLine = lines[i].trim()
          
          // If it's a bullet list item, add it
          if (/^[\*\-\+]\s/.test(currentLine)) {
            const content = currentLine.slice(2)
            listItems.push(`<li class="flex items-start"><span class="text-blue-400 mr-2 mt-1">•</span><span>${content}</span></li>`)
            i++
          }
          // If it's an empty line, skip it but continue looking for more list items
          else if (currentLine === '') {
            i++
          }
          // If it's not a list item and not empty, we're done with this list
          else {
            break
          }
        }
        
        result.push(`<ul class="my-3 space-y-1">${listItems.join('')}</ul>`)
        continue
      }
      
      // Ordered lists (handle empty lines between items)
      if (/^\d+\.\s/.test(trimmed)) {
        const listItems = []
        let listNumber = 1
        
        while (i < lines.length) {
          const currentLine = lines[i].trim()
          
          // If it's a numbered list item, add it
          if (/^\d+\.\s/.test(currentLine)) {
            const content = currentLine.replace(/^\d+\.\s/, '')
            listItems.push(`<li class="flex items-start"><span class="text-green-400 mr-2 mt-1 font-medium min-w-[1.5rem]">${listNumber}.</span><span>${content}</span></li>`)
            listNumber++
            i++
          }
          // If it's an empty line, skip it but continue looking for more list items
          else if (currentLine === '') {
            i++
          }
          // If it's not a list item and not empty, we're done with this list
          else {
            break
          }
        }
        
        result.push(`<ol class="my-3 space-y-1">${listItems.join('')}</ol>`)
        continue
      }
      
      // Tables
      if (trimmed.includes('|') && trimmed.startsWith('|') && trimmed.endsWith('|')) {
        const tableLines = []
        while (i < lines.length && lines[i].trim().includes('|')) {
          const tableLine = lines[i].trim()
          if (tableLine.startsWith('|') && tableLine.endsWith('|')) {
            tableLines.push(tableLine)
          }
          i++
        }
        
        if (tableLines.length > 0) {
          const tableHtml = this.processTable(tableLines)
          result.push(tableHtml)
        }
        continue
      }
      
      // Regular paragraph
      result.push(`<p class="text-white/85 my-2 leading-relaxed">${line}</p>`)
      i++
    }
    
    return result.join('\n')
  }
  
  private static processInlineElements(text: string): string {
    return text
      // Agent mentions (preserve this feature!)
      .replace(/@(enteract|coding|research|vision)\b/g, (match, agent) => {
        const agentStyles = {
          enteract: 'bg-blue-500/20 text-blue-300 border-blue-400/30',
          coding: 'bg-green-500/20 text-green-300 border-green-400/30',
          research: 'bg-purple-500/20 text-purple-300 border-purple-400/30',
          vision: 'bg-pink-500/20 text-pink-300 border-pink-400/30'
        }
        const style = agentStyles[agent as keyof typeof agentStyles] || 'bg-gray-500/20 text-gray-300 border-gray-400/30'
        return `<span class="inline-flex items-center px-2 py-1 rounded-md text-xs font-medium border ${style}">@${agent}</span>`
      })
      
      // Bold and italic (order matters!)
      .replace(/\*\*\*(.*?)\*\*\*/g, '<strong class="font-bold text-white"><em class="italic">$1</em></strong>')
      .replace(/\*\*(.*?)\*\*/g, '<strong class="font-semibold text-white">$1</strong>')
      .replace(/\*(.*?)\*/g, '<em class="italic text-white/90">$1</em>')
      
      // Alternative bold/italic syntax
      .replace(/\_\_\_(.*?)\_\_\_/g, '<strong class="font-bold text-white"><em class="italic">$1</em></strong>')
      .replace(/\_\_(.*?)\_\_/g, '<strong class="font-semibold text-white">$1</strong>')
      .replace(/\_(.*?)\_/g, '<em class="italic text-white/90">$1</em>')
      
      // Links (enhanced with better styling)
      .replace(/\[([^\]]+)\]\(([^)]+)\)/g, '<a href="$2" class="text-blue-400 hover:text-blue-300 underline decoration-blue-400/50 hover:decoration-blue-300 transition-colors" target="_blank" rel="noopener noreferrer">$1 <span class="text-xs">↗</span></a>')
      
      // Strikethrough
      .replace(/~~(.*?)~~/g, '<del class="line-through text-white/60">$1</del>')
      
      // Images
      .replace(/!\[([^\]]*)\]\(([^)]+)\)/g, '<img src="$2" alt="$1" class="max-w-full h-auto rounded-lg my-2" />')
  }
  
  private static processTable(tableLines: string[]): string {
    if (tableLines.length < 2) return ''
    
    const rows = tableLines.map(line => {
      return line.slice(1, -1).split('|').map(cell => cell.trim())
    })
    
    // Check if second row is a separator
    const isSeparator = (row: string[]) => row.every(cell => /^[\-\:\s]*$/.test(cell))
    
    let headerRow = rows[0]
    let dataRows = rows.slice(1)
    
    if (dataRows.length > 0 && isSeparator(dataRows[0])) {
      dataRows = dataRows.slice(1)
    }
    
    const headerHtml = headerRow.map(cell => 
      `<th class="px-4 py-2 border border-white/20 bg-white/5 text-white font-semibold text-left">${cell}</th>`
    ).join('')
    
    const dataHtml = dataRows.map(row => 
      `<tr>${row.map(cell => 
        `<td class="px-4 py-2 border border-white/20 text-white/85">${cell}</td>`
      ).join('')}</tr>`
    ).join('')
    
    return `<div class="table-container overflow-x-auto my-4">
      <table class="min-w-full border-collapse border border-white/20 rounded-lg overflow-hidden">
        <thead><tr>${headerHtml}</tr></thead>
        <tbody>${dataHtml}</tbody>
      </table>
    </div>`
  }
  
  private static highlightCode(code: string, language: string): string {
    if (!code) return ''
    
    // First escape HTML to prevent any issues
    const escapedCode = this.escapeHtml(code)
    
    // Debug logging
    console.log('Highlighting:', { language, originalLength: code.length, escapedLength: escapedCode.length })
    
    // Use a token-based approach for better syntax highlighting
    let result = ''
    switch (language) {
      case 'python':
        result = this.highlightPython(escapedCode)
        break
      case 'javascript':
      case 'js':
        result = this.highlightJavaScript(escapedCode)
        break
      case 'typescript':
      case 'ts':
        result = this.highlightTypeScript(escapedCode)
        break
      case 'rust':
        result = this.highlightRust(escapedCode)
        break
      case 'json':
        result = this.highlightJSON(escapedCode)
        break
      case 'html':
        result = this.highlightHTML(escapedCode)
        break
      case 'css':
        result = this.highlightCSS(escapedCode)
        break
      case 'bash':
      case 'shell':
        result = this.highlightBash(escapedCode)
        break
      default:
        result = `<span class="text-gray-300">${escapedCode}</span>`
    }
    
    console.log('Highlight result length:', result.length)
    return result
  }
  
  // Token-based highlighting approach to avoid regex conflicts
  private static tokenizeAndHighlight(code: string, tokens: Array<{pattern: RegExp, className: string}>): string {
    let result = code
    
    // Process tokens in order, being careful not to break existing highlights
    for (const token of tokens) {
      // Only match if we're not inside an existing span
      result = result.replace(token.pattern, (match, ...groups) => {
        // Check if this match is inside an existing HTML tag
        const beforeMatch = result.substring(0, result.indexOf(match))
        const openTags = (beforeMatch.match(/<span[^>]*>/g) || []).length
        const closeTags = (beforeMatch.match(/<\/span>/g) || []).length
        
        // If we're inside a span, don't highlight
        if (openTags > closeTags) {
          return match
        }
        
        // Apply the highlighting
        if (groups.length > 0) {
          return match.replace(groups[0], `<span class="${token.className}">${groups[0]}</span>`)
        } else {
          return `<span class="${token.className}">${match}</span>`
        }
      })
    }
    
    return result
  }
  
  private static highlightPython(code: string): string {
    // Use a safer, sequential approach
    let result = code
    
    // 1. Triple-quoted strings FIRST (before single line comments to avoid conflicts)
    result = result.replace(/("""[\s\S]*?""")/g, '<span class="text-green-300">$1</span>')
    result = result.replace(/('''[\s\S]*?''')/g, '<span class="text-green-300">$1</span>')
    
    // 2. Single line comments (after docstrings)
    result = result.replace(/(?!<span[^>]*>)(#[^\n]*)(?![^<]*<\/span>)/g, '<span class="text-gray-500 italic">$1</span>')
    
    // 3. Regular strings (only if not already highlighted)
    result = result.replace(/(?!<span[^>]*>)("(?:[^"\\]|\\.)*")(?![^<]*<\/span>)/g, '<span class="text-green-300">$1</span>')
    result = result.replace(/(?!<span[^>]*>)('(?:[^'\\]|\\.)*')(?![^<]*<\/span>)/g, '<span class="text-green-300">$1</span>')
    
    // 4. Keywords (only if not in strings or comments)
    const keywords = ['def', 'class', 'if', 'elif', 'else', 'for', 'while', 'try', 'except', 'finally', 'with', 'as', 'import', 'from', 'return', 'yield', 'break', 'continue', 'pass', 'raise', 'assert', 'global', 'nonlocal', 'lambda', 'and', 'or', 'not', 'in', 'is', 'True', 'False', 'None']
    keywords.forEach(keyword => {
      result = result.replace(new RegExp(`(?!<span[^>]*>)\\b(${keyword})\\b(?![^<]*<\/span>)`, 'g'), '<span class="text-purple-400 font-semibold">$1</span>')
    })
    
    // 5. Built-in functions
    const builtins = ['print', 'len', 'range', 'enumerate', 'zip', 'map', 'filter', 'sum', 'max', 'min', 'sorted', 'reversed', 'any', 'all', 'isinstance', 'type', 'str', 'int', 'float', 'list', 'dict', 'set', 'tuple', 'open', 'input']
    builtins.forEach(builtin => {
      result = result.replace(new RegExp(`(?!<span[^>]*>)\\b(${builtin})(?=\\s*\\()(?![^<]*<\/span>)`, 'g'), '<span class="text-blue-400">$1</span>')
    })
    
    // 6. Numbers
    result = result.replace(/(?!<span[^>]*>)\b(\d+(?:\.\d+)?)\b(?![^<]*<\/span>)/g, '<span class="text-orange-400">$1</span>')
    
    // 7. Decorators
    result = result.replace(/(?!<span[^>]*>)(@\w+)(?![^<]*<\/span>)/g, '<span class="text-pink-400">$1</span>')
    
    // 8. Self
    result = result.replace(/(?!<span[^>]*>)\b(self)\b(?![^<]*<\/span>)/g, '<span class="text-cyan-400">$1</span>')
    
    return result
  }
  
  private static highlightJavaScript(code: string): string {
    let result = code
    
    // 1. Multi-line comments first
    result = result.replace(/(\/\*[\s\S]*?\*\/)/g, '<span class="text-gray-500 italic">$1</span>')
    
    // 2. Single line comments (after multi-line to avoid conflicts)
    result = result.replace(/(?!<span[^>]*>)(\/\/[^\n]*)(?![^<]*<\/span>)/g, '<span class="text-gray-500 italic">$1</span>')
    
    // 2. Strings
    result = result.replace(/(?!<span[^>]*>)("(?:[^"\\]|\\.)*")(?![^<]*<\/span>)/g, '<span class="text-green-300">$1</span>')
    result = result.replace(/(?!<span[^>]*>)('(?:[^'\\]|\\.)*')(?![^<]*<\/span>)/g, '<span class="text-green-300">$1</span>')
    result = result.replace(/(?!<span[^>]*>)(`(?:[^`\\]|\\.)*`)(?![^<]*<\/span>)/g, '<span class="text-green-300">$1</span>')
    
    // 3. Keywords
    const keywords = ['const', 'let', 'var', 'function', 'class', 'if', 'else', 'for', 'while', 'do', 'switch', 'case', 'default', 'try', 'catch', 'finally', 'return', 'break', 'continue', 'throw', 'new', 'this', 'super', 'extends', 'import', 'export', 'from', 'async', 'await', 'typeof', 'instanceof']
    keywords.forEach(keyword => {
      result = result.replace(new RegExp(`(?!<span[^>]*>)\\b(${keyword})\\b(?![^<]*<\/span>)`, 'g'), '<span class="text-purple-400 font-semibold">$1</span>')
    })
    
    // 4. Numbers
    result = result.replace(/(?!<span[^>]*>)\b(\d+(?:\.\d+)?)\b(?![^<]*<\/span>)/g, '<span class="text-orange-400">$1</span>')
    
    // 5. Booleans and null
    const literals = ['true', 'false', 'null', 'undefined']
    literals.forEach(literal => {
      result = result.replace(new RegExp(`(?!<span[^>]*>)\\b(${literal})\\b(?![^<]*<\/span>)`, 'g'), '<span class="text-purple-300">$1</span>')
    })
    
    return result
  }
  
  private static highlightTypeScript(code: string): string {
    let result = code
    
    // Start with JavaScript highlighting
    result = this.highlightJavaScript(result)
    
    // Add TypeScript-specific keywords
    const tsKeywords = ['interface', 'type', 'enum', 'implements', 'public', 'private', 'protected', 'readonly', 'static']
    tsKeywords.forEach(keyword => {
      result = result.replace(new RegExp(`(?!<span[^>]*>)\\b(${keyword})\\b(?![^<]*<\/span>)`, 'g'), '<span class="text-purple-400 font-semibold">$1</span>')
    })
    
    // Type annotations (simplified)
    result = result.replace(/(?!<span[^>]*>)(:\s*)(\w+)(?![^<]*<\/span>)/g, '$1<span class="text-cyan-400">$2</span>')
    
    return result
  }

  private static highlightRust(code: string): string {
    let result = code
    
    // 1. Multi-line comments first
    result = result.replace(/(\/\*[\s\S]*?\*\/)/g, '<span class="text-gray-500 italic">$1</span>')
    
    // 2. Single line comments (after multi-line to avoid conflicts)
    result = result.replace(/(?!<span[^>]*>)(\/\/[^\n]*)(?![^<]*<\/span>)/g, '<span class="text-gray-500 italic">$1</span>')
    
    // 2. Strings
    result = result.replace(/(?!<span[^>]*>)("(?:[^"\\]|\\.)*")(?![^<]*<\/span>)/g, '<span class="text-green-300">$1</span>')
    result = result.replace(/(?!<span[^>]*>)('(?:[^'\\]|\\.)*')(?![^<]*<\/span>)/g, '<span class="text-green-300">$1</span>')
    
    // 3. Keywords
    const keywords = ['fn', 'let', 'mut', 'const', 'struct', 'enum', 'impl', 'trait', 'pub', 'use', 'mod', 'crate', 'super', 'self', 'if', 'else', 'match', 'for', 'while', 'loop', 'break', 'continue', 'return', 'async', 'await', 'unsafe', 'extern', 'static', 'type', 'where']
    keywords.forEach(keyword => {
      result = result.replace(new RegExp(`(?!<span[^>]*>)\\b(${keyword})\\b(?![^<]*<\/span>)`, 'g'), '<span class="text-purple-400 font-semibold">$1</span>')
    })
    
    // 4. Numbers
    result = result.replace(/(?!<span[^>]*>)\b(\d+(?:\.\d+)?)\b(?![^<]*<\/span>)/g, '<span class="text-orange-400">$1</span>')
    
    // 5. Booleans
    result = result.replace(/(?!<span[^>]*>)\b(true|false)\b(?![^<]*<\/span>)/g, '<span class="text-purple-300">$1</span>')
    
    return result
  }
  
  private static highlightJSON(code: string): string {
    let result = code
    
    // 1. Property names
    result = result.replace(/("[\w\s-]+")\s*:/g, '<span class="text-blue-400">$1</span>:')
    
    // 2. String values
    result = result.replace(/:\s*("(?:[^"\\]|\\.)*")/g, ': <span class="text-green-300">$1</span>')
    
    // 3. Boolean and null values
    result = result.replace(/:\s*(true|false|null)\b/g, ': <span class="text-purple-400">$1</span>')
    
    // 4. Numbers
    result = result.replace(/:\s*(\d+\.?\d*)/g, ': <span class="text-orange-400">$1</span>')
    
    return result
  }
  
  private static highlightHTML(code: string): string {
    // HTML is already escaped, so we work with &lt; &gt;
    return code.replace(/&lt;(\/?[\w-]+)([^&]*?)&gt;/g, (match, tag, attrs) => {
      let highlightedAttrs = attrs
      if (attrs.trim()) {
        // Highlight attributes
        highlightedAttrs = attrs.replace(/([\w-]+)=(["'])([^"']*)\2/g, 
          '<span class="text-blue-400">$1</span>=<span class="text-green-300">$2$3$2</span>')
      }
      return `&lt;<span class="text-purple-400">${tag}</span>${highlightedAttrs}&gt;`
    })
  }
  
  private static highlightCSS(code: string): string {
    let result = code
    
    // 1. Selectors
    result = result.replace(/([.#]?[\w-]+)\s*\{/g, '<span class="text-yellow-400">$1</span> {')
    
    // 2. Properties and values
    result = result.replace(/([\w-]+):\s*([^;]+);/g, '<span class="text-blue-400">$1</span>: <span class="text-green-300">$2</span>;')
    
    return result
  }
  
  private static highlightBash(code: string): string {
    let result = code
    
    // 1. Strings first (before comments to avoid conflicts)
    result = result.replace(/(?!<span[^>]*>)("(?:[^"\\]|\\.)*")(?![^<]*<\/span>)/g, '<span class="text-green-300">$1</span>')
    result = result.replace(/(?!<span[^>]*>)('(?:[^'\\]|\\.)*')(?![^<]*<\/span>)/g, '<span class="text-green-300">$1</span>')
    
    // 2. Comments (after strings)
    result = result.replace(/(?!<span[^>]*>)(#[^\n]*)(?![^<]*<\/span>)/g, '<span class="text-gray-500 italic">$1</span>')
    
    // 3. Prompt symbols
    result = result.replace(/^(\$|#)\s*/gm, '<span class="text-gray-500">$1</span> ')
    
    // 4. Common commands
    const commands = ['cd', 'ls', 'mkdir', 'rm', 'cp', 'mv', 'grep', 'find', 'awk', 'sed', 'cat', 'echo', 'export', 'sudo', 'chmod', 'chown']
    commands.forEach(cmd => {
      result = result.replace(new RegExp(`(?!<span[^>]*>)\\b(${cmd})\\b(?![^<]*<\/span>)`, 'g'), '<span class="text-purple-400">$1</span>')
    })
    
    return result
  }
  
  private static escapeHtml(text: string): string {
    return text
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
      .replace(/"/g, '&quot;')
      .replace(/'/g, '&#39;')
  }
}