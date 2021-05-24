const colors = require('tailwindcss/colors');

module.exports = {
  purge: ['./index.html', './src/**/*.{vue,js,ts,jsx,tsx}'],
  darkMode: false, // or 'media' or 'class'
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
        ...colors.gray,
        darkest: '#050509',
        dark: '#0a0a12',
        200: '#0e0e1a',
        250: '#11111f',
        300: '#1a1a1f',
        350: '#212124',
        400: '#2c2c2e',
        500: '#3b3b3d',
      },
      black: colors.black,
      white: colors.white,
      yellow: colors.amber,
      pink: colors.pink,
    },
    fontFamily: {
      serif: ['"Zilla Slab"', 'serif'],
      sans: ['Lato' ,'Helvetica', 'Arial', 'sans-serif'],
    },
    boxShadow: {
      light: '0 4px 6px -1px #fff2, 0 2px 4px -1px #fff1'
    },
    minWidth: {
      '10rem': '10rem',
      '15rem': '15rem',
      '20rem': '20rem',
      '1/3': '33.3333%',
      '60p': '60%',
      '40p': '40%',
      '30vw': '30vw',
      '40vw': '40vw'
    },
      transitionTimingFunction: {
        'hyper-out': 'cubic-bezier(0.000, 0.875, 0.065, 1.000)',
        'hyper-in': 'cubic-bezier(0.650, 0.000, 1.000, 0.095)'
      },
  },
  variants: {
    extend: {
      borderWidth: ['hover'],
      backgroundColor: ['disabled'],
      cursor: ['disabled'],
    },
  },
  plugins: [],
}
