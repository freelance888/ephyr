<script lang="ts">
  export let id: string;
  export let checked = false;
  export let classes = '';
  export let title = '';

  export let confirmFn: (onSuccess: () => void) => void;
  export let onChangeFn: () => unknown;

  let domElement: HTMLInputElement;

  function onChange() {
    if (confirmFn) {
      const currentValue = domElement.checked;
      const successFn = () => {
        domElement.checked = currentValue;
        if (onChangeFn) onChangeFn();
      };

      confirmFn(successFn);
      domElement.checked = !currentValue;
    } else {
      if (onChangeFn) onChangeFn();
    }
  }
</script>

<template>
  <span
    class="toggle {classes}"
    {title}
    style="font-size:{classes.includes('small') ? 8 : 10}px"
  >
    <input
      type="checkbox"
      bind:checked
      bind:this={domElement}
      {id}
      on:click={onChange}
    />
    <label for={id} class="toggle" />
  </span>
</template>

<style lang="stylus" global>
  .toggle
    input[type="checkbox"]
      display: none

      & + label
        cursor: pointer
        position: relative
        display: inline-block
        height: 2em
        width: 4em
        background: #cecece
        border-radius: 20em
        vertical-align: middle
        transition: .3s background
        user-select: none
        margin-top: -5px

        &, &::before, &::after
          box-sizing: border-box

        &::before, &::after
          position: absolute
          top: 0
          left: 0
        &::before
          content: ''
          display: inline-block
          background: rgba(0, 0, 0, .5)
          height: 1.5em
          width: 1.5em
          margin: .25em
          border-radius: 50%
          z-index: 200
          transition:.3s left
        &::after
          color: #fff
          content: '\f00d'
          font: 1.25em 'Font Awesome 5 Pro'
          top: 50%
          margin-top: -.5em
          left: 2em

      &:checked + label
        background: #08c

        &::before
          left: 2em
        &::after
          content: '\f00c'
          left: .5em

    &.small
      input[type="checkbox"]
        & + label
          margin-top: -4px
</style>
