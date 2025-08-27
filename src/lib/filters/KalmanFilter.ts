export interface KalmanConfig {
  processNoise: number
  measurementNoise: number
  initialState: { x: number; y: number }
  initialVelocity: { vx: number; vy: number }
}

export class KalmanFilter2D {
  private x: number
  private y: number
  private vx: number
  private vy: number
  private P: number[][]
  private Q: number
  private R: number

  constructor(config: KalmanConfig) {
    this.x = config.initialState.x
    this.y = config.initialState.y
    this.vx = config.initialVelocity.vx
    this.vy = config.initialVelocity.vy
    this.Q = config.processNoise
    this.R = config.measurementNoise
    
    // Initialize covariance matrix
    this.P = [
      [1, 0, 0, 0],
      [0, 1, 0, 0],
      [0, 0, 1, 0],
      [0, 0, 0, 1]
    ]
  }

  predict(dt: number): void {
    // State transition
    this.x += this.vx * dt
    this.y += this.vy * dt
    
    // Update covariance matrix
    const dt2 = dt * dt
    const dt3 = dt2 * dt
    const dt4 = dt3 * dt
    
    this.P[0][0] += dt4 * this.Q / 4
    this.P[0][1] += dt3 * this.Q / 2
    this.P[0][2] += dt3 * this.Q / 2
    this.P[0][3] += dt2 * this.Q
    
    this.P[1][0] += dt3 * this.Q / 2
    this.P[1][1] += dt2 * this.Q
    this.P[1][2] += dt2 * this.Q
    this.P[1][3] += dt * this.Q
    
    this.P[2][0] += dt3 * this.Q / 2
    this.P[2][1] += dt2 * this.Q
    this.P[2][2] += dt2 * this.Q
    this.P[2][3] += dt * this.Q
    
    this.P[3][0] += dt2 * this.Q
    this.P[3][1] += dt * this.Q
    this.P[3][2] += dt * this.Q
    this.P[3][3] += this.Q
  }

  update(measurement: { x: number; y: number }): void {
    // Kalman gain calculation
    const S = this.P[0][0] + this.R
    const Kx = this.P[0][0] / S
    const Ky = this.P[1][1] / S
    
    // Update state
    const innovationX = measurement.x - this.x
    const innovationY = measurement.y - this.y
    
    this.x += Kx * innovationX
    this.y += Ky * innovationY
    
    // Update covariance
    const I_Kx = 1 - Kx
    const I_Ky = 1 - Ky
    
    this.P[0][0] *= I_Kx
    this.P[1][1] *= I_Ky
  }

  getPosition(): { x: number; y: number } {
    return { x: this.x, y: this.y }
  }

  getVelocity(): { vx: number; vy: number } {
    return { vx: this.vx, vy: this.vy }
  }

  reset(config: KalmanConfig): void {
    this.x = config.initialState.x
    this.y = config.initialState.y
    this.vx = config.initialVelocity.vx
    this.vy = config.initialVelocity.vy
    this.Q = config.processNoise
    this.R = config.measurementNoise
    
    this.P = [
      [1, 0, 0, 0],
      [0, 1, 0, 0],
      [0, 0, 1, 0],
      [0, 0, 0, 1]
    ]
  }
}
