/**
 * Kalman Filter for 2D Gaze Coordinate Smoothing
 * Implements a standard Kalman filter to reduce jitter in eye tracking data
 */

export interface KalmanConfig {
  // Process noise covariance (how much we trust the model)
  processNoise: number
  // Measurement noise covariance (how much we trust the measurements)  
  measurementNoise: number
  // Initial error covariance
  initialError: number
}

export interface KalmanState {
  // State vector [x, y, vx, vy] - position and velocity
  state: number[]
  // Error covariance matrix
  errorCovariance: number[][]
}

export class KalmanFilter2D {
  private config: KalmanConfig
  private state: KalmanState
  private isInitialized = false

  // State transition matrix (constant velocity model)
  private F: number[][] = [
    [1, 0, 1, 0], // x = x + vx*dt
    [0, 1, 0, 1], // y = y + vy*dt  
    [0, 0, 1, 0], // vx = vx
    [0, 0, 0, 1]  // vy = vy
  ]

  // Observation matrix (we observe position only)
  private H: number[][] = [
    [1, 0, 0, 0], // observe x
    [0, 1, 0, 0]  // observe y
  ]

  // Process noise covariance matrix
  private Q: number[][]

  // Measurement noise covariance matrix  
  private R: number[][]

  constructor(config: Partial<KalmanConfig> = {}) {
    this.config = {
      processNoise: config.processNoise || 0.1,
      measurementNoise: config.measurementNoise || 1.0,
      initialError: config.initialError || 100.0
    }

    // Initialize noise matrices
    this.Q = this.createProcessNoiseMatrix(this.config.processNoise)
    this.R = this.createMeasurementNoiseMatrix(this.config.measurementNoise)

    this.state = {
      state: [0, 0, 0, 0], // [x, y, vx, vy]
      errorCovariance: this.createIdentityMatrix(4, this.config.initialError)
    }
  }

  /**
   * Initialize the filter with the first measurement
   */
  initialize(x: number, y: number): void {
    this.state.state = [x, y, 0, 0] // Start with zero velocity
    this.state.errorCovariance = this.createIdentityMatrix(4, this.config.initialError)
    this.isInitialized = true
  }

  /**
   * Update the filter with a new measurement
   */
  update(x: number, y: number, deltaTime: number = 1.0): { x: number, y: number } {
    if (!this.isInitialized) {
      this.initialize(x, y)
      return { x, y }
    }

    // Update time step in state transition matrix
    this.F[0][2] = deltaTime
    this.F[1][3] = deltaTime

    // Prediction step
    this.predict()

    // Correction step
    this.correct([x, y])

    return {
      x: this.state.state[0],
      y: this.state.state[1]
    }
  }

  /**
   * Prediction step: predict the next state
   */
  private predict(): void {
    // Predict state: x_k = F * x_{k-1}
    this.state.state = this.multiplyMatrixVector(this.F, this.state.state)

    // Predict error covariance: P_k = F * P_{k-1} * F^T + Q
    const FP = this.multiplyMatrices(this.F, this.state.errorCovariance)
    const FPFt = this.multiplyMatrices(FP, this.transpose(this.F))
    this.state.errorCovariance = this.addMatrices(FPFt, this.Q)
  }

  /**
   * Correction step: correct prediction with measurement
   */
  private correct(measurement: number[]): void {
    // Calculate Kalman gain: K = P * H^T * (H * P * H^T + R)^-1
    const Ht = this.transpose(this.H)
    const PH = this.multiplyMatrices(this.state.errorCovariance, Ht)
    const HPHt = this.multiplyMatrices(this.H, PH)
    const S = this.addMatrices(HPHt, this.R)
    const Sinv = this.invertMatrix2x2(S)
    const K = this.multiplyMatrices(PH, Sinv)

    // Update state: x = x + K * (z - H * x)
    const Hx = this.multiplyMatrixVector(this.H, this.state.state)
    const innovation = this.subtractVectors(measurement, Hx)
    const correction = this.multiplyMatrixVector(K, innovation)
    this.state.state = this.addVectors(this.state.state, correction)

    // Update error covariance: P = (I - K * H) * P
    const I = this.createIdentityMatrix(4)
    const KH = this.multiplyMatrices(K, this.H)
    const IminusKH = this.subtractMatrices(I, KH)
    this.state.errorCovariance = this.multiplyMatrices(IminusKH, this.state.errorCovariance)
  }

  /**
   * Reset the filter to initial state
   */
  reset(): void {
    this.isInitialized = false
    this.state = {
      state: [0, 0, 0, 0],
      errorCovariance: this.createIdentityMatrix(4, this.config.initialError)
    }
  }

  /**
   * Update filter configuration
   */
  updateConfig(config: Partial<KalmanConfig>): void {
    this.config = { ...this.config, ...config }
    this.Q = this.createProcessNoiseMatrix(this.config.processNoise)
    this.R = this.createMeasurementNoiseMatrix(this.config.measurementNoise)
  }

  /**
   * Get current velocity estimate
   */
  getVelocity(): { vx: number, vy: number } {
    return {
      vx: this.state.state[2],
      vy: this.state.state[3]
    }
  }

  /**
   * Get current position estimate
   */
  getPosition(): { x: number, y: number } {
    return {
      x: this.state.state[0],
      y: this.state.state[1]
    }
  }

  // Matrix utility functions
  private createIdentityMatrix(size: number, scale: number = 1): number[][] {
    const matrix = Array(size).fill(0).map(() => Array(size).fill(0))
    for (let i = 0; i < size; i++) {
      matrix[i][i] = scale
    }
    return matrix
  }

  private createProcessNoiseMatrix(noise: number): number[][] {
    const dt = 1.0 // Time step
    const dt2 = dt * dt
    const dt3 = dt2 * dt
    const dt4 = dt3 * dt

    return [
      [dt4/4 * noise, 0, dt3/2 * noise, 0],
      [0, dt4/4 * noise, 0, dt3/2 * noise],
      [dt3/2 * noise, 0, dt2 * noise, 0],
      [0, dt3/2 * noise, 0, dt2 * noise]
    ]
  }

  private createMeasurementNoiseMatrix(noise: number): number[][] {
    return [
      [noise, 0],
      [0, noise]
    ]
  }

  private multiplyMatrices(A: number[][], B: number[][]): number[][] {
    const rows = A.length
    const cols = B[0].length
    const result = Array(rows).fill(0).map(() => Array(cols).fill(0))
    
    for (let i = 0; i < rows; i++) {
      for (let j = 0; j < cols; j++) {
        for (let k = 0; k < A[0].length; k++) {
          result[i][j] += A[i][k] * B[k][j]
        }
      }
    }
    return result
  }

  private multiplyMatrixVector(matrix: number[][], vector: number[]): number[] {
    return matrix.map(row => 
      row.reduce((sum, val, i) => sum + val * vector[i], 0)
    )
  }

  private addMatrices(A: number[][], B: number[][]): number[][] {
    return A.map((row, i) => row.map((val, j) => val + B[i][j]))
  }

  private subtractMatrices(A: number[][], B: number[][]): number[][] {
    return A.map((row, i) => row.map((val, j) => val - B[i][j]))
  }

  private addVectors(a: number[], b: number[]): number[] {
    return a.map((val, i) => val + b[i])
  }

  private subtractVectors(a: number[], b: number[]): number[] {
    return a.map((val, i) => val - b[i])
  }

  private transpose(matrix: number[][]): number[][] {
    return matrix[0].map((_, i) => matrix.map(row => row[i]))
  }

  private invertMatrix2x2(matrix: number[][]): number[][] {
    const [[a, b], [c, d]] = matrix
    const det = a * d - b * c
    
    if (Math.abs(det) < 1e-10) {
      // Matrix is singular, return identity
      return [[1, 0], [0, 1]]
    }
    
    return [
      [d / det, -b / det],
      [-c / det, a / det]
    ]
  }
} 