<script lang="ts">
  import Fa from 'svelte-fa';
  import { faClose } from '@fortawesome/free-solid-svg-icons';
  import { faCheck } from '@fortawesome/free-solid-svg-icons';

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
    <label for={id} class="toggle">
      <Fa class={!checked ? 'uk-invisible' : ''} icon={faCheck} scale={1.2} />
      <Fa class={checked ? 'uk-invisible' : ''} icon={faClose} scale={1.2} />
    </label>
  </span>
</template>

<style lang="stylus" global>
  .toggle
    display: inline-block
    input[type="checkbox"]
      display: none

      & + label
        cursor: pointer
        position: relative
        display: flex
        align-items: center
        justify-content: space-around
        height: 2em
        width: 4em
        background: #cecece
        border-radius: 20em
        transition: .3s background
        user-select: none
        margin-top: -5px

        svg
          color: #fff

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

      &:checked + label
        background: #08c

        &::before
          left: 2em

    &.small
      input[type="checkbox"]
        & + label
          margin-top: 3px

</style>
