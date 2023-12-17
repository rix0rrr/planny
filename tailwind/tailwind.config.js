/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
      'templates/**/*.html',
  ],
  input: 'base.css',
  output: 'static/css/styles.css',
  theme: {
    extend: {},
  },
  plugins: [],
}

