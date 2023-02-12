<script lang="js">
  import { createEventDispatcher } from 'svelte'
  const dispatch = createEventDispatcher()

 let items = [
  {
   type: 'error',
   message: 'Can\'t create vertical or horizontal splits you only need the Split component.'
  },
  {
   type: 'warning',
   message: 'The default creates a left(50%) | right(50%) split, no minimum pane sizes, and renders the default splitter.'
  },
  {
   type: 'info',
   message: 'The Switcher component consists of a number of toggles and their related content items. Add the uk-switcher attribute to a list element which contains the toggles. Add the .uk-switcher class to the element containing the content items.'
  },
 ]

  let isOpen = false;

  const dispatchToggleConsole = () => {
    isOpen = !isOpen;
    dispatch('toggleConsole', isOpen)
  };

</script>

<template>
 <section>
  <div class='console-toolbar uk-flex' on:click={dispatchToggleConsole}>
    <span class='console-btn'>Console</span>
    <a class='clear-btn uk-margin-auto-left'>Clear ({items.length})</a>
  </div>

  <ul class="messages-container uk-list uk-list-divider uk-list-collapse">
    {#each items as item}
     <li
       class:uk-text-danger={item.type === 'error'}
       class:uk-text-warning={item.type === 'warning'}
     >
      { item.message }
     </li>
    {/each}
  </ul>
 </section>
</template>

<style lang="stylus">

 section
   padding: 8px 16px 16px 16px

 .console-toolbar
  cursor: pointer

 .messages-container
  margin-top: 8px;

 .console-btn
  font-weight: bold
</style>
