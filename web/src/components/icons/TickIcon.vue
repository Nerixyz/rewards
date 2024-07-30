<template>
  <div class="tick-outer m-auto">
    <svg class="tick-svg" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 10 10">
      <path
        class="tick-path text-red"
        fill="transparent"
        d="M 1 6 c 0.662 1.117 1.602 2.977 3 3 c 1.813 -0.023 4.031 -6.032 5.116 -7.817"
      />
    </svg>
  </div>
</template>

<style scoped>
.tick-outer {
  width: 60px;
  height: 60px;
}
.tick-svg {
  --tick-duration: 500ms;

  width: 100%;
  height: 100%;
  margin: auto;
  stroke-width: 1px;
  transform: scale(1);
  opacity: 1;
}
.tick-path {
  stroke: currentColor;
  stroke-linecap: round;
  animation: tick-animation var(--tick-duration) cubic-bezier(0.46, 0.005, 0.355, 1) forwards;
  transform-origin: center;
}
@keyframes tick-animation {
  from {
    stroke-dasharray: 0, 200%;
    stroke-dashoffset: 0;
    transform: scale(0.7);
    opacity: 0;
  }
  to {
    stroke-dasharray: 200%, 200%;
    stroke-dashoffset: -150%;
    transform: scale(1);
    opacity: var(--full-opacity, 1);
  }
}
</style>
