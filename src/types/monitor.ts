export interface MonitorInfo {
  x: number
  y: number
  width: number
  height: number
  is_primary: boolean
  name: string
}

export interface CalibrationPoint {
  monitor_index: number
  target_x: number
  target_y: number
  gaze_x?: number
  gaze_y?: number
  timestamp?: number
}

export interface CalibrationState {
  active: boolean
  current_target: number
  total_targets: number
  points: CalibrationPoint[]
  completed: boolean
  skipped: boolean
} 