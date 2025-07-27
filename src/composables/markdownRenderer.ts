// markdownRenderer.ts - Enhanced markdown renderer with agent mention support
export class MarkdownRenderer {
    static render(text: string): string {
      if (!text) return ''
      
      return text
        // Agent mentions (before other formatting)
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
        
        // Headers
        .replace(/^### (.*$)/gim, '<h3 class="text-lg font-semibold text-white/90 mt-4 mb-2">$1</h3>')
        .replace(/^## (.*$)/gim, '<h2 class="text-xl font-semibold text-white/95 mt-4 mb-2">$1</h2>')
        .replace(/^# (.*$)/gim, '<h1 class="text-2xl font-bold text-white mt-4 mb-3">$1</h1>')
        
        // Bold and italic
        .replace(/\*\*(.*?)\*\*/g, '<strong class="font-semibold text-white">$1</strong>')
        .replace(/\*(.*?)\*/g, '<em class="italic text-white/90">$1</em>')
        
        // Code blocks with syntax highlighting hints
        .replace(/```(\w+)?\n?([\s\S]*?)```/g, (match, lang, code) => {
          const language = lang || 'text'
          const langClass = this.getLanguageClass(language)
          return `<div class="bg-black/30 border border-white/20 rounded-lg p-3 my-2 font-mono text-sm overflow-x-auto">
            <div class="text-xs text-white/60 mb-2 uppercase tracking-wide">${language}</div>
            <div class="${langClass}">${code}</div>
          </div>`
        })
        .replace(/`(.*?)`/g, '<code class="bg-black/40 px-1.5 py-0.5 rounded text-sm font-mono text-cyan-300">$1</code>')
        
        // Enhanced lists with better styling
        .replace(/^\* (.*$)/gim, '<li class="ml-4 text-white/85 flex items-start"><span class="text-blue-400 mr-2">•</span><span>$1</span></li>')
        .replace(/^- (.*$)/gim, '<li class="ml-4 text-white/85 flex items-start"><span class="text-blue-400 mr-2">•</span><span>$1</span></li>')
        .replace(/^\+ (.*$)/gim, '<li class="ml-4 text-white/85 flex items-start"><span class="text-blue-400 mr-2">•</span><span>$1</span></li>')
        .replace(/^\d+\. (.*$)/gim, (match, content, offset, string) => {
          const lines = string.substring(0, offset).split('\n')
          const currentLine = lines[lines.length - 1]
          const number = currentLine.match(/^(\d+)\./)?.[1] || '1'
          return `<li class="ml-4 text-white/85 flex items-start"><span class="text-green-400 mr-2 font-medium">${number}.</span><span>${content}</span></li>`
        })
        
        // Enhanced links with better styling
        .replace(/\[([^\]]+)\]\(([^)]+)\)/g, '<a href="$2" class="text-blue-400 hover:text-blue-300 underline decoration-blue-400/50 hover:decoration-blue-300 transition-colors" target="_blank" rel="noopener noreferrer">$1 <span class="text-xs">↗</span></a>')
        
        // Blockquotes
        .replace(/^> (.*$)/gim, '<blockquote class="border-l-4 border-blue-500/50 pl-4 italic text-white/80 my-2">$1</blockquote>')
        
        // Tables (basic support)
        .replace(/\|(.+)\|/g, (match) => {
          const cells = match.split('|').filter(cell => cell.trim()).map(cell => 
            `<td class="px-3 py-2 border border-white/20 text-white/85">${cell.trim()}</td>`
          ).join('')
          return `<tr>${cells}</tr>`
        })
        
        // Horizontal rules
        .replace(/^---$/gm, '<hr class="border-white/20 my-4" />')
        
        // Line breaks
        .replace(/\n\n/g, '<br/><br/>')
        .replace(/\n/g, '<br/>')
    }
    
    private static getLanguageClass(language: string): string {
      const languageClasses: Record<string, string> = {
        javascript: 'text-yellow-300',
        typescript: 'text-blue-300',
        python: 'text-green-300',
        rust: 'text-orange-300',
        go: 'text-cyan-300',
        java: 'text-red-300',
        cpp: 'text-purple-300',
        c: 'text-purple-300',
        html: 'text-orange-300',
        css: 'text-pink-300',
        json: 'text-yellow-300',
        xml: 'text-orange-300',
        sql: 'text-blue-300',
        bash: 'text-green-300',
        shell: 'text-green-300',
        powershell: 'text-blue-300',
        yaml: 'text-purple-300',
        markdown: 'text-gray-300',
        text: 'text-gray-300'
      }
      
      return languageClasses[language.toLowerCase()] || 'text-gray-300'
    }
  }