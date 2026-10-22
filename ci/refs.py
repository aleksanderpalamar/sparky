def event_base(name, event):
    if name == "pull_request":
        return event["pull_request"]["base"]["sha"]
    if name == "merge_group":
        return event["merge_group"]["base_sha"]
    if name != "push":
        return None
    previous = event.get("before", "")
    return previous if previous and set(previous) != {"0"} else None


def pull_request_problem(event):
    request = event.get("pull_request")
    if request is None:
        return None
    if request["head"]["ref"] != request["base"]["ref"]:
        return None
    return "O PR precisa de uma branch própria, diferente da branch de destino."
