# Enteract Frontend Testing Suite

This directory contains comprehensive tests for the Enteract Tauri application's frontend/UI components using Vitest and Vue Test Utils.

## Test Structure

```
src/tests/
├── components/          # Component unit and integration tests
│   ├── ControlPanel.test.ts
│   ├── ChatWindow.test.ts
│   ├── ConversationalWindow.test.ts
│   ├── ControlPanelButtons.test.ts
│   └── WindowInteractions.test.ts
├── composables/         # Composable function tests
│   ├── useWindowManager.test.ts
│   └── useSpeechTranscription.test.ts
├── utils/              # Utility function tests
├── __mocks__/          # Mock implementations
│   ├── tauri.ts
│   └── composables.ts
├── setup.ts            # Test setup and global mocks
└── README.md           # This file
```

## Test Categories

### 1. Component Tests
- **ControlPanel.test.ts**: Tests the main control panel component rendering and functionality
- **ChatWindow.test.ts**: Tests chat window interactions, input handling, and message display
- **ConversationalWindow.test.ts**: Tests conversational interface, microphone controls, and recording
- **ControlPanelButtons.test.ts**: Integration tests for control panel button functionality
- **WindowInteractions.test.ts**: Cross-component integration tests for window management

### 2. Composable Tests
- **useWindowManager.test.ts**: Tests window state management and operations
- **useSpeechTranscription.test.ts**: Tests speech recognition functionality and state

### 3. Integration Tests
Tests that verify multiple components working together:
- Window opening/closing workflows
- Button click → window state changes
- Keyboard shortcuts
- Multi-window interactions

## Running Tests

### Basic Test Commands
```bash
# Run all tests
npm run test

# Run tests with UI
npm run test:ui

# Run tests with coverage report
npm run test:coverage

# Run tests once (CI mode)
npm run test:run
```

### Test Filtering
```bash
# Run only component tests
npm run test components

# Run specific test file
npm run test ControlPanel

# Run tests matching pattern
npm run test window
```

## Test Features

### Mocking Strategy
- **Tauri APIs**: All `@tauri-apps/api` calls are mocked to prevent actual system calls
- **Speech Recognition**: Browser speech APIs are mocked for consistent testing
- **Composables**: Mock implementations provide predictable behavior
- **Icons**: Heroicons are stubbed to avoid rendering issues

### Test Coverage
The test suite covers:
- ✅ Component rendering and structure
- ✅ User interactions (clicks, input, keyboard)
- ✅ State management and reactivity
- ✅ Event emissions and prop handling
- ✅ Window lifecycle management
- ✅ Speech transcription workflows
- ✅ Error handling and edge cases
- ✅ Cross-platform compatibility within Tauri context

### Cross-Platform Testing
Tests are designed to work consistently across:
- Windows (primary target)
- macOS (Tauri support)
- Linux (Tauri support)

## Best Practices

### Writing New Tests
1. **Arrange-Act-Assert**: Structure tests clearly
2. **Mock Dependencies**: Use provided mocks for external dependencies
3. **Test User Behavior**: Focus on what users actually do
4. **Avoid Implementation Details**: Test public APIs, not internals
5. **Use Descriptive Names**: Test names should explain the scenario

### Example Test Structure
```typescript
describe('ComponentName', () => {
  let wrapper: any

  beforeEach(() => {
    wrapper = mount(ComponentName, {
      props: { /* required props */ },
      global: {
        stubs: { /* child components */ }
      }
    })
  })

  describe('Feature Group', () => {
    it('should handle specific user action', async () => {
      // Arrange
      const button = wrapper.find('[data-testid="button"]')
      
      // Act
      await button.trigger('click')
      
      // Assert
      expect(wrapper.emitted('event')).toBeTruthy()
    })
  })
})
```

## Continuous Integration

Tests are designed to run in CI environments with:
- Headless browser environment (happy-dom)
- Consistent mock behaviors
- No external dependencies
- Fast execution times

## Troubleshooting

### Common Issues
1. **Component not rendering**: Check that all required props are provided
2. **Mock not working**: Verify mock is imported before the component
3. **Async test failing**: Use `await wrapper.vm.$nextTick()` after state changes
4. **Event not emitted**: Check event name spelling and trigger method

### Debug Tips
```typescript
// Log component HTML
console.log(wrapper.html())

// Check component data
console.log(wrapper.vm.$data)

// Verify emitted events
console.log(wrapper.emitted())
```

## Future Enhancements

- [ ] Visual regression testing
- [ ] Performance benchmarking
- [ ] E2E tests with real Tauri environment
- [ ] Accessibility testing
- [ ] Error boundary testing