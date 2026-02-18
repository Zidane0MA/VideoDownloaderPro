import { render, screen } from '@testing-library/react';
import { describe, it, expect } from 'vitest';

describe('Example Test', () => {
  it('should render correctly', () => {
    render(<div data-testid="test-div">Hello Vitest</div>);
    expect(screen.getByTestId('test-div')).toHaveTextContent('Hello Vitest');
  });
});
