<script lang="js">
  import { mutation } from 'svelte-apollo';

  import { SetEndpointLabel } from '../../api/client.graphql';
  import { saveOrCloseByKeys } from '../utils/directives.util';

  import { showError } from '../utils/util';

  const setLabelMutation = mutation(SetEndpointLabel);

  export let endpoint;
  export let input;
  export let restream_id;

  let label_component;
  let label_input;
  let show_edit = false;

  async function showEdit() {
    show_edit = true;
  }
  async function hideEdit() {
    show_edit = false;
  }
  async function cancelEdit() {
    label_component.value = endpoint.label;
    hideEdit();
  }
  async function submit() {
    const variables = {
      restream_id: restream_id,
      input_id: input.id,
      endpoint_id: endpoint.id,
      label: label_input.value,
    };
    try {
      let result_val = await setLabelMutation({ variables });
      if (result_val.data.setEndpointLabel) {
        endpoint.label = label_input.value;
        label_component.value = endpoint.label;
        await hideEdit();
      } else {
        showError('Provided text has invalid characters or is too long.');
      }
    } catch (e) {
      showError(e.message);
    }
  }

  function init_input(label_input) {
    label_input.value = endpoint.label;
    label_input.focus();
  }
</script>

<template>
  <div class="endpoint-label">
    <span bind:this={label_component} class:hidden={show_edit}
      >{endpoint.label ? endpoint.label : ''}</span
    >
    {#if show_edit}
      <input
        bind:this={label_input}
        use:init_input
        use:saveOrCloseByKeys={{
          save: submit,
          close: cancelEdit,
        }}
      />
    {/if}
    <a
      class="edit-label"
      href="/"
      on:click|preventDefault={() => {
        showEdit();
      }}
    >
      <i class="far fa-edit" title="Edit label" />
    </a>
  </div>
</template>

<style lang="stylus">
  .endpoint-label
    margin-left 5px
    color: #999

    &:hover
      .edit-label
        opacity: 1

    .hidden
      display: none

    .edit-label
      opacity: 0
      transition: opacity .3s ease
      color: #666
      outline: none
      &:hover
        opacity: 1
        text-decoration: none
        color: #444
</style>
