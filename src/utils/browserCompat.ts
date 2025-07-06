// Browser compatibility utilities for speech features

export interface BrowserSupport {
  speechRecognition: boolean
  mediaDevices: boolean
  getUserMedia: boolean
  webRTC: boolean
}

export interface BrowserInfo {
  name: string
  version: string
  isSupported: boolean
  recommendation: string
}

export const checkBrowserSupport = (): BrowserSupport => {
  const support: BrowserSupport = {
    speechRecognition: 'SpeechRecognition' in window || 'webkitSpeechRecognition' in window,
    mediaDevices: 'mediaDevices' in navigator,
    getUserMedia: !!(navigator.mediaDevices && navigator.mediaDevices.getUserMedia),
    webRTC: !!(window.RTCPeerConnection || (window as any).webkitRTCPeerConnection || (window as any).mozRTCPeerConnection)
  }
  
  return support
}

export const getBrowserInfo = (): BrowserInfo => {
  const userAgent = navigator.userAgent
  let name = 'Unknown'
  let version = 'Unknown'
  let isSupported = false
  let recommendation = 'For best results, use Chrome or Edge'
  
  if (userAgent.includes('Chrome') && !userAgent.includes('Edge')) {
    name = 'Chrome'
    isSupported = true
    recommendation = 'Chrome detected - excellent compatibility for speech features'
    
    const match = userAgent.match(/Chrome\/(\d+)/)
    if (match) {
      version = match[1]
    }
  } else if (userAgent.includes('Edge')) {
    name = 'Edge'
    isSupported = true
    recommendation = 'Edge detected - good compatibility for speech features'
    
    const match = userAgent.match(/Edge\/(\d+)/)
    if (match) {
      version = match[1]
    }
  } else if (userAgent.includes('Firefox')) {
    name = 'Firefox'
    isSupported = false
    recommendation = 'Firefox has limited speech API support. Use Chrome or Edge for best experience'
    
    const match = userAgent.match(/Firefox\/(\d+)/)
    if (match) {
      version = match[1]
    }
  } else if (userAgent.includes('Safari') && !userAgent.includes('Chrome')) {
    name = 'Safari'
    isSupported = false
    recommendation = 'Safari has limited speech API support. Use Chrome or Edge for best experience'
    
    const match = userAgent.match(/Version\/(\d+)/)
    if (match) {
      version = match[1]
    }
  }
  
  return {
    name,
    version,
    isSupported,
    recommendation
  }
}

export const getSpeechFeatureSupport = () => {
  const support = checkBrowserSupport()
  const browserInfo = getBrowserInfo()
  
  return {
    ...support,
    browser: browserInfo,
    overallSupport: support.speechRecognition && support.getUserMedia,
    warnings: [
      !support.speechRecognition && 'Speech Recognition API not supported',
      !support.mediaDevices && 'MediaDevices API not supported',
      !support.getUserMedia && 'getUserMedia not supported',
      !browserInfo.isSupported && browserInfo.recommendation
    ].filter(Boolean)
  }
}

export const checkSecureContext = (): boolean => {
  return window.isSecureContext || location.protocol === 'https:' || location.hostname === 'localhost'
}

export const getCompatibilityReport = () => {
  const speechSupport = getSpeechFeatureSupport()
  const isSecure = checkSecureContext()
  
  return {
    ...speechSupport,
    isSecureContext: isSecure,
    ready: speechSupport.overallSupport && isSecure,
    issues: [
      ...speechSupport.warnings,
      !isSecure && 'HTTPS or localhost required for microphone access'
    ].filter(Boolean)
  }
} 