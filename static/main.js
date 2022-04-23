const e = React.createElement;

// Props: id, onClick, label
class Button extends React.Component
{
    constructor(props)
    {
        super(props);
    }

    render()
    {
        return e("a", {"className": "Button", "id": this.props["id"],
                       "href": "#", "onClick": this.props["onClick"]},
                 this.props["label"]);
    }
}

// Props: title
class Title extends React.Component
{
    constructor(props)
    {
        super(props);
    }

    render()
    {
        return e("h2", {"className": "SectionTitle"}, this.props["title"]);
    }
}

// Props: data
class OngoingJobView extends React.Component
{
    constructor(props)
    {
        super(props);
    }

    render()
    {
        return e("li", null, this.props["data"]["uri"]);
    }
}

// Props: jobs
class OngoingJobList extends React.Component
{
    constructor(props)
    {
        super(props);
    }

    render()
    {
        let items = this.props["jobs"].map(j =>
            e(OngoingJobView, {"data": j, "key": j["uri"]}));
        return e("section", null,
                 e(Title, {"title": "Ongoing Jobs"}),
                 e("ul", {"id": "OngoingJobList", "className": "JobList"},
                   items));
    }
}

// Props: data
class StoppedJobView extends React.Component
{
    constructor(props)
    {
        super(props);
    }

    render()
    {
        let status_text = "✅";
        let status_tooltip = "Finished";
        if(this.props["data"]["stop_reason"] != "Done")
        {
            status_text = "❌";
            status_tooltip = JSON.stringify(this.props["data"]["stop_reason"]["Error"]);
        }

        return e("li", null,
                 e("span", {"className": "JobURI"}, this.props["data"]["uri"]),
                 e("span", {"className": "JobStatus", "title": status_tooltip},
                   status_text));
    }
}

// Props: jobs
class StoppedJobList extends React.Component
{
    constructor(props)
    {
        super(props);
    }

    render()
    {
        let items = this.props["jobs"].map(j =>
            e(StoppedJobView, {"data": j, "key": j["uri"]}));
        return e("section", null,
                 e(Title, {"title": "Stopped Jobs"}),
                 e("ul", {"id": "StoppedJobList", "className": "JobList"},
                   items));
    }
}

// Props: onClickNewJob
class JobAdderView extends React.Component
{
    constructor(props)
    {
        super(props);
    }

    render()
    {
        return e("section", null,
                 e(Title, {"title": "Add Download"}),
                 e("div", {"className": "ControlRow"},
                   e("input", {"type": "text", "id": "URIInput"}),
                   e(Button, {"id": "BtnAddJob",
                              "onClick": this.props["onClickNewJob"],
                              "label": "Download!"})));
    }
}

class MainView extends React.Component
{
    constructor(props)
    {
        super(props);
        this.state = {
            "jobs_ongoing": [],
            "jobs_stopped": [],
        };
        this.onClickNewJob = this.onClickNewJob.bind(this);
    }

    onClickNewJob()
    {
        let uri = document.getElementById("URIInput").value;
        if(!uri) return;
        fetch("api/new_job",
              {method: 'POST',
               headers: {'Content-Type': 'application/json'},
               body: JSON.stringify({"uri": uri})})
            .then(response => response.json())
            .then(data => {
                console.log(data);
                this.fetchJobs();
            });
    }

    fetchJobs()
    {
        fetch('api/jobs')
            .then(response => response.json())
            .then(data => this.setState(
                {"jobs_ongoing": data["ongoing"],
                 "jobs_stopped": data["stopped"],}));
    }

    componentDidMount()
    {
        this.fetchJobs();
    }

    render()
    {
        return e("div", null,
                 e(JobAdderView, {"onClickNewJob": this.onClickNewJob}),
                 e(OngoingJobList, {"jobs": this.state["jobs_ongoing"]}),
                 e(StoppedJobList, {"jobs": this.state["jobs_stopped"]}));
    }
}

const dom_container = document.querySelector('#Main');
ReactDOM.render(e(MainView, null), dom_container);
