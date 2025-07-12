// markdownRenderer.ts - Simple markdown renderer for basic formatting
export class MarkdownRenderer {
    static render(text: string): string {
      if (!text) return ''
      
      return text
        // Headers
        .replace(/^### (.*$)/gim, '<h3 class="text-lg font-semibold text-white/90 mt-4 mb-2">$1</h3>')
        .replace(/^## (.*$)/gim, '<h2 class="text-xl font-semibold text-white/95 mt-4 mb-2">$1</h2>')
        .replace(/^# (.*$)/gim, '<h1 class="text-2xl font-bold text-white mt-4 mb-3">$1</h1>')
        
        // Bold and italic
        .replace(/\*\*(.*?)\*\*/g, '<strong class="font-semibold text-white">$1</strong>')
        .replace(/\*(.*?)\*/g, '<em class="italic text-white/90">$1</em>')
        
        // Code blocks
        .replace(/```([\s\S]*?)```/g, '<div class="bg-black/30 border border-white/20 rounded-lg p-3 my-2 font-mono text-sm text-green-300 overflow-x-auto">$1</div>')
        .replace(/`(.*?)`/g, '<code class="bg-black/40 px-1.5 py-0.5 rounded text-sm font-mono text-cyan-300">$1</code>')
        
        // Lists
        .replace(/^\* (.*$)/gim, '<li class="ml-4 text-white/85">• $1</li>')
        .replace(/^- (.*$)/gim, '<li class="ml-4 text-white/85">• $1</li>')
        .replace(/^\+ (.*$)/gim, '<li class="ml-4 text-white/85">• $1</li>')
        .replace(/^\d+\. (.*$)/gim, '<li class="ml-4 text-white/85">$1</li>')
        
        // Links
        .replace(/\[([^\]]+)\]\(([^)]+)\)/g, '<a href="$2" class="text-blue-400 hover:text-blue-300 underline" target="_blank" rel="noopener noreferrer">$1</a>')
        
        // Line breaks
        .replace(/\n\n/g, '<br/><br/>')
        .replace(/\n/g, '<br/>')
    }
  }