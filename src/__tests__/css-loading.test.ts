import { describe, it, expect } from 'vitest';
import { readFileSync } from 'fs';
import { resolve } from 'path';

describe('CSS 로딩 테스트', () => {

  it('style.css 파일이 존재해야 함', () => {
    const stylePath = resolve(__dirname, '../style.css');
    const styleContent = readFileSync(stylePath, 'utf-8');
    expect(styleContent).toBeTruthy();
  });

  it('Tailwind CSS import가 포함되어 있어야 함', () => {
    const stylePath = resolve(__dirname, '../style.css');
    const styleContent = readFileSync(stylePath, 'utf-8');
    expect(styleContent).toContain('@import "tailwindcss"');
  });

  it('@theme 커스텀 테마가 정의되어 있어야 함', () => {
    const stylePath = resolve(__dirname, '../style.css');
    const styleContent = readFileSync(stylePath, 'utf-8');
    expect(styleContent).toContain('@theme');
    expect(styleContent).toContain('--font-family-sans');
  });

  it('CSS 변수(custom properties)가 정의되어 있어야 함', () => {
    const stylePath = resolve(__dirname, '../style.css');
    const styleContent = readFileSync(stylePath, 'utf-8');
    expect(styleContent).toContain('--color-primary');
    expect(styleContent).toContain('--color-success');
    expect(styleContent).toContain('--color-error');
  });
});

describe('PostCSS 설정 테스트', () => {
  it('postcss.config.js가 존재해야 함', () => {
    const configPath = resolve(__dirname, '../../postcss.config.js');
    const configContent = readFileSync(configPath, 'utf-8');
    expect(configContent).toBeTruthy();
  });

  it('PostCSS 설정에 @tailwindcss/postcss가 포함되어야 함', () => {
    const configPath = resolve(__dirname, '../../postcss.config.js');
    const configContent = readFileSync(configPath, 'utf-8');
    expect(configContent).toContain('@tailwindcss/postcss');
  });

  it('PostCSS 설정에 autoprefixer가 포함되어야 함', () => {
    const configPath = resolve(__dirname, '../../postcss.config.js');
    const configContent = readFileSync(configPath, 'utf-8');
    expect(configContent).toContain('autoprefixer');
  });
});

describe('Tailwind 클래스 사용 테스트', () => {
  it('App.vue에서 Tailwind 클래스를 사용해야 함', () => {
    const appPath = resolve(__dirname, '../App.vue');
    const appContent = readFileSync(appPath, 'utf-8');

    // 주요 Tailwind 클래스 확인
    expect(appContent).toContain('min-h-screen');
    expect(appContent).toContain('bg-gray-100');
    expect(appContent).toContain('flex');
  });
});

describe('빌드 CSS 출력 테스트', () => {
  it('main.ts에서 style.css를 import해야 함', () => {
    const mainPath = resolve(__dirname, '../main.ts');
    const mainContent = readFileSync(mainPath, 'utf-8');
    expect(mainContent).toMatch(/import ['"]\.\/style\.css['"]/);
  });
});
