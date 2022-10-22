// Remember theme selection
const themeSwitch = document.querySelector('.theme-switch');
themeSwitch.checked = localStorage.getItem('darkTheme') === 'true';

// if localStorage.getItem('darkTheme') === 'true' {
// 	document.querySelector('.theme-switch').checked = true;
// }

themeSwitch.addEventListener('change', function (e) {
  if(e.currentTarget.checked === true) {
    // Add item to localstorage
    localStorage.setItem('darkTheme', 'true');
  } else {
    // Remove item if theme is switched back to normal
    localStorage.removeItem('darkTheme');
  }
});