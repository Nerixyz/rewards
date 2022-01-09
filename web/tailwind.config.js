const colors = require('tailwindcss/colors');

module.exports = {
  content: ['./index.html', './src/**/*.{vue,js,ts,jsx,tsx}'],
  media: false, // or 'media' or 'class'
  theme: {
    colors: {
      transparent: 'transparent',
      current: 'currentColor',
      blue: {
        ...colors.blue,
        light: '#7ad4ff',
        DEFAULT: '#1fb6ff',
        dark: '#009eeb',
      },
      red: {
        ...colors.red,
        light: '#ff5b69',
        DEFAULT: '#ff4151',
        dark: '#d71d29',
      },
      gray: {
        darkest: '#050509',
        dark: '#0a0a12',
        200: '#0e0e1a',
        250: '#11111f',
        300: '#1a1a1f',
        350: '#212124',
        400: '#2c2c2e',
        500: '#3b3b3d',
        600: '#71717a',
        700: '#a1a1aa',
        800: '#d4d4d8',
        900: '#e4e4e7',
        1000: '#f4f4f5',
      },
      black: colors.black,
      white: colors.white,
      yellow: colors.amber,
      pink: colors.pink,
    },
    fontFamily: {
      serif: ['"Zilla Slab"', 'serif'],
      sans: ['Lato', 'Helvetica', 'Arial', 'sans-serif'],
      mono: ['ui-monospace', 'monospace'],
    },
    boxShadow: {
      light: '0 4px 6px -1px #fff2, 0 2px 4px -1px #fff1',
    },
    minWidth: {
      '10rem': '10rem',
      '15rem': '15rem',
      '20rem': '20rem',
      '1/3': '33.3333%',
      '60p': '60%',
      '40p': '40%',
      '30vw': '30vw',
      '40vw': '40vw',
    },
    maxHeight: {
      '75vh': '75vh',
    },
    transitionTimingFunction: {
      'hyper-out': 'cubic-bezier(0.000, 0.875, 0.065, 1.000)',
      'hyper-in': 'cubic-bezier(0.650, 0.000, 1.000, 0.095)',
      'cubic-out': 'cubic-bezier(0.215, 0.610, 0.355, 1.000)',
    },
    gridTemplateColumns: {
      logs: 'auto 1fr',
    },
    keyframes: {
      bing: {
        '0%, 100%': { transform: 'scale(100%) rotate(0)' },
        '50%': { transform: 'scale(105%) rotate(20deg)' },
      },
    },
    animation: {
      bing: 'bing 1s ease-out infinite',
    },
  },
  plugins: [],
};
