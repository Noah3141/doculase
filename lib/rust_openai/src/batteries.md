Look at this essay and generate JSON of the following format:

{"duality": /_ determine a float value 0.0-1.0 that estimates the degree to which the text treats both sides of the argument with equal understanding _/}

This worked kind of well; silly statements in a paragraph really brought down the score.

Take this review paper and generate this JSON, where values are a float 0.0-1.0:
{
"hypothesis_presence": /_ degree to which the paper concerns itself with hypotheses _/ ,
"mechanism_presence": /_ degree to which the paper concerns itself with mechanisms of action _/ ,
"balance": /_ degree to which the paper gives representing opposing perspectives equal effort _/
}

MM/DD/YYYY does not always work to prompt it do provide the date in that format

Temperature recommendations by Battery:
